# cloud-config

创建链的配置文件和部署文件。

### 依赖

* rust: 1.59
* libsqlite3

### 安装

```
cargo install --path .
```

### 用法

```
$ cloud-config -h
cloud-config 6.4.0
Rivtower Technologies <contact@rivtower.com>

USAGE:
    cloud-config <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    append-dev           append node in env dev
    append-k8s           append node in env k8s
    append-node          append a node into chain
    append-validator     append a validator into chain
    create-ca            create CA
    create-csr           create csr
    create-dev           create config in env dev
    create-k8s           create config in env k8s
    delete-chain         delete a chain
    delete-dev           delete node in env dev
    delete-k8s           delete node in env k8s
    delete-node          delete a node from chain
    delete-validator     delete a validator from chain
    help                 Print this message or the help of the given subcommand(s)
    import-account       import account
    import-ca            import ca
    import-cert          import node cert
    init-chain           init a chain
    init-chain-config    init chain config
    init-node            init node
    migrate              migrate CITA-Cloud chain from 6.1.0 to 6.3.0
    new-account          new account
    set-admin            set admin of chain
    set-nodelist         set node list
    set-stage            set stage
    set-validators       set validators of chain
    sign-csr             sign csr
    update-node          update node
    update-yaml          update k8s yaml
```

### 设计
先前工具的实现方式是，通过用户传递的命令行参数，直接生成一条链所有节点的微服务配置文件。

这种实现方式有两个问题：

1. 所有的逻辑混在一起。当需要针对不同的运行环境设置不同的配置逻辑时，会增加很多的分支判断，导致代码可维护性下降。比如，针对开发环境，需要在本地运行所有节点，因此需要各个节点的微服务端口不重复；而针对`k8s`生产环境，则要求各个节点的微服务端口保持一致。
2. 因为要同时传递一条链多个节点的信息，用户需要传递的参数非常多，心智负担比较大，非常容易出错。并且也不符合实际生产环境部署的时候，各个节点由不同参与方来提供信息的特点。

在新的实现中，抽象出[链级配置](/src/config/chain_config.rs)和[节点配置](/src/config/node_config.rs)两个数据结构。`链级配置`包含一条链内所有节点都需要的公共信息；`节点配置`包含链内一个具体节点所需的信息。两个数据结构结合，包含了创建一条链所需的所有信息。

用户通过命令行参数传递的信息来填充这两个数据结构，然后据此产生出各个节点的所有配置文件。这样就分离了前端信息填充和后端生成配置文件。后续不管是优化前端用户输入信息的体验，还是在以后版本中配置文件结构有变化，都会比较容易修改，保证代码可维护性比较高。

用户通过命令行参数填充`链级配置`和`节点配置`的过程也被拆分成了多个子命令。一方面可以减少单个子命令的参数个数，另一方面整个过程更加灵活。可以适用于测试时集中生成的中心化模式，也可以适用于实际生产部署时，不同子命令由不同参与方执行的去中心化模式。

#### 配置项与数据结构

当前版本的配置项如下图所示：
![configuration](/img/configurations.png)

数据结构如下图所示：
![struct](/img/struct.png)

#### 流程

将创建链配置的过程拆分成10个子命令和7个辅助的子命令，以实现最正规的去中心化配置流程。

7个辅助子命令为：

1. [create-ca](/src/create_ca.rs)创建链的根证书。会在`$(config-dir)/$(chain-name)/ca_cert/`下生成`cert.pem`和`key.pem`两个文件。
2. [create-csr](/src/create_csr.rs)为各个节点创建证书和签名请求。会在`$(config-dir)/$(chain-name)/certs/$(domain)/`下生成`csr.pem`和`key.pem`两个文件。
3. [sign-csr](/src/sign_csr.rs)处理节点的签名请求。会在`$(config-dir)/$(chain-name)/certs/$(domain)/`下生成`cert.pem`。
4. [new-account](/src/new_account.rs)创建账户。会在`$(config-dir)/$(chain-name)/accounts/`下，创建以账户地址为名的文件夹，里面有`key_id`和`kms.db`两个文件。
5. [import-account](/src/import_account.rs)导入私钥的方式创建账户。
6. [import-ca](/src/import_ca.rs)导入已有的`CA`证书。要求证书格式为`pem`，`key`的格式为`pkcs8`。
7. [import-cert](/src/import_cert.rs)导入已有的节点证书。要求证书格式为`pem`，`key`的格式为`pkcs8`。

10个子命令分别为：
1. [init-chain](/src/init_chain.rs)。根据指定的`config-dir`和`chan-name`,初始化一个链的文件目录结构。
    ```
    $(config_dir)
    --  $(chain_name)
    ------  accounts
    --------  .gitkeep
    ------  ca_cert
    --------  .gitkeep
    ------  certs
    --------  .gitkeep
    ------  .gitignore
    ```
2. [init-chain-config](/src/init_chain_config.rs)。初始化除`admin`(管理员账户)，`validators`(共识节点地址列表)，`node_network_address_list`（节点网络地址列表）之外的`链级配置`。因为前述三个操作需要一些额外的准备工作，且需要先对除此之外的链接配置信息在所有参与方之间达成共识。因此对于去中心化场景来说，这一步其实是一个公示的过程。执行之后会生成`$(config-dir)/$(chain-name)/chain_config.toml`
3. [set-admin](/src/set_admin.rs)。设置管理员账户。账户需要事先通过[new-account](/src/new_account.rs)子命令创建。如果网络微服务选择了`network_tls`，则还需要通过[create-ca](/src/create_ca.rs)创建链的根证书。
4. [set-validators](/src/set_validators.rs)。设置共识节点账户列表。账户同样需要事先通过[new-account](/src/new_account.rs)子命令，由各个共识节点分别创建，然后将账户地址集中到一起进行设置。
5. [set-nodelist](/src/set_nodelist.rs)。设置节点网络地址列表。各个节点参与方需要根据自己的网络环境，预先保留节点的`ip`，`port`和`domain`。然后将相关信息集中到一起进行设置。至此，`链级配置`信息设置完成，可以下发配置文件`chain_config.toml`到各个节点。如果网络微服务选择了`network_tls`，则需要通过`create-csr`根据节点的`domain`为各个节点创建证书和签名请求。然后请求`CA`通过`sign-crs`处理签名请求，并下发生成的`cert.pem`到各个节点。
6. [set-stage](/src/set_stage.rs)设置链配置进行到的阶段。将链的配置过程分为三个阶段，分别为`init`，`public`和`finalize`，一些子命令会根据当前所处的阶段，来判断是否可以进行操作，以避免一些误操作。`init-chain-config`初始化链的配置时，默认设置阶段为`init`，该阶段可以重复进行`init-chain-config`以修改链的基础配置；`set-admin`只能在`init`阶段执行，并且会自动将阶段修改为`public`；`append-validator`，`delete-validator`和`set-validators`只能在`public`阶段进行；此后需要本子命令将阶段修改为`finalize`，此时链的配置完成，不能再修改除`节点网络地址列表`之外的配置，并且此后才可以进行`init-node`操作。辅助命令和`delete-chain`不受阶段的限制。
7. [init-node](/src/init_node.rs)。设置`节点配置`信息。这步操作由各个节点的参与方独立设置，节点之间可以不同。执行之后会生成`$(config-dir)/$(chain-name)-$(domain)/node_config.toml`
8. [update-node](/src/update_node.rs)。根据之前设置的`链级配置`和`节点配置`，生成每个节点所需的微服务配置文件。
9. [update-yaml](/src/update_yaml.rs)。根据之前设置的`链级配置`和`节点配置`，生成每个节点部署到`k8s`环境所需的资源文件。
10. [delete-chain](/src/delete_chain.rs)删除链。删除属于该链的所有文件夹以及其中的文件，`使用时要慎重`。

此外还有一些其他操作的子命令。

比如增加单个共识节点的`append-validator`，删除单个共识节点的`delete-validator`；增加单个网络节点的`append-node`，删除单个网络节点的`delete-node`。

在前述流程的基础上，封装了更高级更方便使用的子命令。

比如针对开发环境，有`create-dev`，`append-dev`和`delete-dev`，使用开发环境约定好的默认值，无需执行这么多子命令，也无需传递大量参数就可以生成和操作一条链的配置。

针对演示或者生产阶段的`k8s`环境，有`create-k8s`，`append-k8s`和`delete-k8s`。

针对版本升级时迁移配置文件的`migrate`。


### 使用示例

#### init-chain

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
```

```
$ cloud-config init-chain   
$ tree test-chain 
test-chain
├── accounts
├── ca_cert
└── certs
```

#### init-chain-config

参数：

```
        --block_interval <BLOCK_INTERVAL>
            set system config block_interval [default: 3]

        --block_limit <BLOCK_LIMIT>
            set system config block_limit [default: 100]

        --chain-name <CHAIN_NAME>
            set chain name [default: test-chain]

        --chain_id <CHAIN_ID>
            set system config chain_id [default: ]

        --config-dir <CONFIG_DIR>
            set config file directory, default means current directory [default: .]

        --consensus_image <CONSENSUS_IMAGE>
            set consensus micro service image name (consensus_bft/consensus_raft) [default:
            consensus_bft]

        --consensus_tag <CONSENSUS_TAG>
            set consensus micro service image tag [default: latest]

        --controller_image <CONTROLLER_IMAGE>
            set controller micro service image name (controller) [default: controller]

        --controller_tag <CONTROLLER_TAG>
            set controller micro service image tag [default: latest]

        --executor_image <EXECUTOR_IMAGE>
            set executor micro service image name (executor_evm) [default: executor_evm]

        --executor_tag <EXECUTOR_TAG>
            set executor micro service image tag [default: latest]

        --kms_image <KMS_IMAGE>
            set kms micro service image name (kms_eth/kms_sm) [default: kms_sm]

        --kms_tag <KMS_TAG>
            set kms micro service image tag [default: latest]

        --network_image <NETWORK_IMAGE>
            set network micro service image name (network_tls/network_p2p) [default: network_tls]

        --network_tag <NETWORK_TAG>
            set network micro service image tag [default: latest]

        --prevhash <PREVHASH>
            set genesis prevhash [default:
            0x0000000000000000000000000000000000000000000000000000000000000000]

        --storage_image <STORAGE_IMAGE>
            set storage micro service image name (storage_rocksdb) [default: storage_rocksdb]

        --storage_tag <STORAGE_TAG>
            set storage micro service image tag [default: latest]

        --timestamp <TIMESTAMP>
            set genesis timestamp [default: 0]

        --version <VERSION>
            set system config version [default: 0]
```

说明：
1. 参数部分基本对应`链级配置`数据结构，具体含义参见设计部分的描述。


```
$ cloud-config init-chain-config

$ tree test-chain
test-chain
├── accounts
├── ca_cert
├── certs
└── chain_config.toml

$ cat test-chain/chain_config.toml
node_network_address_list = []
stage = 'Init'

[[micro_service_list]]
image = 'network_tls'
tag = 'latest'

[[micro_service_list]]
image = 'consensus_bft'
tag = 'latest'

[[micro_service_list]]
image = 'executor_evm'
tag = 'latest'

[[micro_service_list]]
image = 'storage_rocksdb'
tag = 'latest'

[[micro_service_list]]
image = 'controller'
tag = 'latest'

[[micro_service_list]]
image = 'kms_sm'
tag = 'latest'

[genesis_block]
prevhash = '0x0000000000000000000000000000000000000000000000000000000000000000'
timestamp = 1649418157966

[system_config]
admin = ''
block_interval = 3
block_limit = 100
chain_id = '63586a3c0255f337c77a777ff54f0040b8c388da04f23ecee6bfd4953a6512b4'
validators = []
version = 0
```

说明：
1. `timestamp`默认参数为0。检测到为默认值时，自动替换为当前时间对应的时间戳。
2. `chain_id`默认为空字符串。检测到为默认值时，自动替换为`hex(sm3($(chain_name)))`。

#### new-account

参数：

```
        --chain-name <CHAIN_NAME>
            set chain name [default: test-chain]

        --config-dir <CONFIG_DIR>
            set config file directory, default means current directory [default: .]

        --kms-password <KMS_PASSWORD>
            kms db password [default: 123456]
```

```
$ cloud-config new-account   
key_id:1, address:aeaa6e333b8ed911f89acd01e88e3d9892da87b5

$ cloud-config new-account
key_id:1, address:1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa

$ cloud-config new-account
key_id:1, address:344a9d7c390ea5f884e7c0ebf30abb17bd8785cd

$ tree test-chain 
test-chain
├── accounts
│   ├── 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa
│   │   ├── key_id
│   │   └── kms.db
│   ├── 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd
│   │   ├── key_id
│   │   └── kms.db
│   └── aeaa6e333b8ed911f89acd01e88e3d9892da87b5
│       ├── key_id
│       └── kms.db
├── ca_cert
├── certs
└── chain_config.toml
```

#### import-account
导入账户（私钥）

参数：
```
--chain-name <CHAIN_NAME>
    set chain name [default: test-chain]

--config-dir <CONFIG_DIR>
    set config file directory, default means current directory [default: .]

--kms-password <KMS_PASSWORD>
    kms db password [default: 123456]

--privkey <PRIVKEY>
    hex encoded private key
```

示例：

```
# 将账户私钥导入链的配置
$ cloud-config import-account --chain-name test-chain --config-dir . --km
s-password 123456 --privkey 0xd8e3c336dd0c656e08e8860fd653e2e4a7ff8a8094ca8a36d4bd04facc0741f6
key_id:1, address:f24aeda3a262e81507148d41a9eb9efd1489142c

# 在链的配置中查看这个账户
$ ls test-chain/accounts/f24aeda3a262e81507148d41a9eb9efd1489142c/
key_id  kms.db
```

#### set-admin

参数：

```
        --admin <ADMIN>              set admin
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
```

说明：

1. `admin`为必选参数。值为之前用`new-account`创建的地址。


```
$ cloud-config set-admin --admin aeaa6e333b8ed911f89acd01e88e3d9892da87b5

$ cat test-chain/chain_config.toml | grep admin
admin = 'aeaa6e333b8ed911f89acd01e88e3d9892da87b5'
```

#### set-validators

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --validators <VALIDATORS>    validators account splited by ','
```

说明：

1. `validators`为必选参数。值为多个之前用`new-account`创建的地址,用逗号分隔。

```
$ cloud-config set-validators --validators 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa,344a9d7c390ea5f884e7c0ebf30abb17bd8785cd

$ cat test-chain/chain_config.toml | grep -A3 validators
validators = [
    '1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa',
    '344a9d7c390ea5f884e7c0ebf30abb17bd8785cd',
]
```

#### append-validator

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --validator <VALIDATOR>      validator account
```

说明：

1. `validator`为必选参数。值为之前用`new-account`创建的地址。
2. 功能与`set-validators`相似，只不过是每次添加一个地址。

#### delete-validator

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --validator <VALIDATOR>      validator account
```

说明：

1. `validator`为必选参数。要删除的共识账户地址。
2. 功能与`append-validator`相反，删除一个共识账户。

#### set-stage

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --stage <STAGE>              set stage init/public/finalize [default: finalize]
```

说明：

1. `stage`为要设置的阶段名称。默认值为`finalize`，因为其他两个阶段会随着子命令自动变迁，正常情况下只有`finalize`需要手工设置。各阶段的详细含义参见设计部分的描述。
2. 只有确定过了某个阶段才发现前一个阶段的信息还需要修改，才需要回溯阶段。随意回溯阶段，可能会导致配置被不合理的修改，请谨慎操作。

```
$ cat test-chain/chain_config.toml | grep stage
stage = 'Public'
$ cloud-config set-stage
$ cat test-chain/chain_config.toml | grep stage
stage = 'Finalize'
```

#### set-nodelist

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --nodelist <NODE_LIST>       node list looks like
                                     localhost:40000:node0:k8s_cluster1:40000,localhost:40001:node1:k8s_cluster2
                                     last slice is optional, none means not k8s env
```

说明：

1. `nodelist`为必选参数。值为多个节点的网络地址,用逗号分隔。每个节点的网络地址包含`ip`,`port`，`domain`，`cluster name`，之间用分号分隔。
2. `cluster name`是节点所在的`k8s`集群的名称。出于兼容性考虑，该项可以省略。
3. `domain`为任意字符串，只需要确保节点之间不重复即可。

```
$ cloud-config set-nodelist --nodelist localhost:40000:node0,localhost:40001:node1 

$ cat test-chain/chain_config.toml | grep -A5 node_network
[[node_network_address_list]]
cluster = 'aaIOppLx'
domain = 'node0'
host = 'localhost'
port = 40000
svc_port = 40000
--
[[node_network_address_list]]
cluster = 'xwIaqkcX'
domain = 'node1'
host = 'localhost'
port = 40001
svc_port = 40000
```

```
$ cloud-config set-nodelist --nodelist localhost:40000:node0:k8s_cluster1,localhost:40001:node1:k8s_cluster2
$ cat test-chain/chain_config.toml | grep -A5 node_network
[[node_network_address_list]]
cluster = 'k8s_cluster1'
domain = 'node0'
host = 'localhost'
port = 40000
svc_port = 40000
--
[[node_network_address_list]]
cluster = 'k8s_cluster2'
domain = 'node1'
host = 'localhost'
port = 40001
svc_port = 40000
```

#### append-node

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --node <NODE>                node network address looks like
                                     localhost:40002:node2:k8s_cluster1 last slice is optional, none
                                     means not k8s env
```

1. `node`为必选参数。值为节点的网络地址,包含`ip`,`port`，`domain`，`cluster name`。
2. `cluster name`是节点所在的`k8s`集群的名称。出于兼容性考虑，该项可以省略。
3. 功能与`set-nodelist`相似，只不过是每次添加一个节点。

#### delete-node

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --domain <DOMAIN>            domain of node that want to delete
```

1. `domain`为必选参数。作为节点的标识，标识要删除的节点。
2. 功能与`append-node`相反，删除一个节点。

#### create-ca

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
```

```
$ cloud-config create-ca

$ tree test-chain
test-chain
├── accounts
│   ├── 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa
│   │   ├── key_id
│   │   └── kms.db
│   ├── 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd
│   │   ├── key_id
│   │   └── kms.db
│   └── aeaa6e333b8ed911f89acd01e88e3d9892da87b5
│       ├── key_id
│       └── kms.db
├── ca_cert
│   ├── cert.pem
│   └── key.pem
├── certs
└── chain_config.toml
```

说明：
1. 该命令生成文件形式的根证书，存放在`ca_cert`目录下。

#### import-ca

参数：

```
        --ca-cert <CA_CERT_PATH>     set path of ca cert file(pem)
        --ca-key <CA_KEY_PATH>       set path of ca key file(pem)
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
```

说明：
1. `ca-cert`为必选参数。为要导入的`CA`证书文件路径，格式为`pem`。
2. `ca-key`为必选参数。为要导入的`CA`证书`key`文件路径，格式为`pem`，并且编码格式为`pkcs8`。

#### import-cert

参数：

```
        --cert <CERT_PATH>           set path of cert file(pem)
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --domain <DOMAIN>            domain of node
        --key <KEY_PATH>             set path of key file(pem)
```

说明：
1. `domain`为必选参数。值为前面`set-nodelist`或者`append-node`时传递的节点的网络地址中的`domain`。
2. `cert`为必选参数。为要导入的节点证书文件路径，格式为`pem`。
4. `key`为必选参数。为要导入的节点证书`key`文件路径，格式为`pem`，并且编码格式为`pkcs8`。

#### create-csr

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --domain <DOMAIN>            domain of node
```

说明：
1. `domain`为必选参数。值为前面`set-nodelist`或者`append-node`时传递的节点的网络地址中的`domain`。

```
$ cloud-config create-csr --domain node0 

$ cloud-config create-csr --domain node1

$ tree test-chain
test-chain
├── accounts
│   ├── 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa
│   │   ├── key_id
│   │   └── kms.db
│   ├── 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd
│   │   ├── key_id
│   │   └── kms.db
│   └── aeaa6e333b8ed911f89acd01e88e3d9892da87b5
│       ├── key_id
│       └── kms.db
├── ca_cert
│   ├── cert.pem
│   └── key.pem
├── certs
│   ├── node0
│   │   ├── csr.pem
│   │   └── key.pem
│   └── node1
│       ├── csr.pem
│       └── key.pem
└── chain_config.toml
```

说明：
1. 该命令生成节点的私钥和签名请求，存放在`certs`目录下。
2. 每个节点的文件都在以节点`domain`为文件名的子文件夹内。

#### sign-csr

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --domain <DOMAIN>            domain of node
```

说明：
1. `domain`为必选参数。值为前面执行`create-csr`时节点的`domain`。

```
$ cloud-config sign-csr --domain node0 

$ cloud-config sign-csr --domain node1

$ tree test-chain
test-chain
├── accounts
│   ├── 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa
│   │   ├── key_id
│   │   └── kms.db
│   ├── 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd
│   │   ├── key_id
│   │   └── kms.db
│   └── aeaa6e333b8ed911f89acd01e88e3d9892da87b5
│       ├── key_id
│       └── kms.db
├── ca_cert
│   ├── cert.pem
│   └── key.pem
└── certs
│    ├── node0
│    │   ├── cert.pem
│    │   ├── csr.pem
│    │   └── key.pem
│    └── node1
│        ├── cert.pem
│        ├── csr.pem
│        └── key.pem
└── chain_config.toml
```

说明：
1. 该命令生成节点的证书文件`cert.pem`，存放在`certs`目录下。
2. 每个节点的文件都在以节点`domain`为文件名的子文件夹内。


#### init-node

参数：

```
        --account <ACCOUNT>
            account of node

        --chain-name <CHAIN_NAME>
            set chain name [default: test-chain]

        --config-dir <CONFIG_DIR>
            set config file directory, default means current directory [default: .]

        --consensus-port <CONSENSUS_PORT>
            grpc consensus_port of node [default: 50001]

        --controller-port <CONTROLLER_PORT>
            grpc controller_port of node [default: 50004]

        --domain <DOMAIN>
            domain of node

        --executor-port <EXECUTOR_PORT>
            grpc executor_port of node [default: 50002]

        --key-id <KEY_ID>
            key id of account in kms db [default: 1]

        --kms-password <KMS_PASSWORD>
            kms db password [default: 123456]

        --kms-port <KMS_PORT>
            grpc kms_port of node [default: 50005]

        --log-level <LOG_LEVEL>
            log level [default: info]

        --network-listen-port <NETWORK_LISTEN_PORT>
            network listen port of node [default: 40000]

        --network-port <NETWORK_PORT>
            grpc network_port of node [default: 50000]

        --package-limit <PACKAGE_LIMIT>
            set one block contains tx limit, default 30000 [default: 30000]

        --storage-port <STORAGE_PORT>
            grpc storage_port of node [default: 50003]
```

说明：
1. 参数部分基本对应`节点配置`数据结构，具体含义参见设计部分的描述。
2. `domain`为必选参数，作为节点的标识，节点文件夹将会以`$(chanin-name)-$(domain)`的形式命名。
3. `account`为必选参数，表示该节点要使用的账户地址。值为之前用`new-account`创建的地址。

```
$ cloud-config init-node --domain node0 --account 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa
 
$ cloud-config init-node --domain node1 --account 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd

$ tree test-chain-node* 
test-chain-node0
├── accounts
│   ├── 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa
│   │   ├── key_id
│   │   └── kms.db
│   ├── 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd
│   │   ├── key_id
│   │   └── kms.db
│   └── aeaa6e333b8ed911f89acd01e88e3d9892da87b5
│       ├── key_id
│       └── kms.db
├── ca_cert
├── certs
├── chain_config.toml
└── node_config.toml
test-chain-node1
├── accounts
│   ├── 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa
│   │   ├── key_id
│   │   └── kms.db
│   ├── 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd
│   │   ├── key_id
│   │   └── kms.db
│   └── aeaa6e333b8ed911f89acd01e88e3d9892da87b5
│       ├── key_id
│       └── kms.db
├── ca_cert
├── certs
├── chain_config.toml
└── node_config.toml


$ cat test-chain-node0/node_config.toml
account = '1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa'
db_key = '123456'
key_id = 1
log_level = 'info'
network_listen_port = 40000
package_limit = 30000

[grpc_ports]
consensus_port = 50001
controller_port = 50004
executor_port = 50002
kms_port = 50005
network_port = 50000
storage_port = 50003
```

#### update-node

参数：

```
        --chain-name <CHAIN_NAME>      set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>      set config file directory, default means current directory
                                       [default: .]
        --config-name <CONFIG_NAME>    set config file name [default: config.toml]
        --domain <DOMAIN>              domain of node
        --is-old                       is old node
        --is-stdout                    is output to stdout
```

说明：
1. `domain`为必选参数，作为节点的标识，表示要操作的节点。
2. `is-old`用于区别是刷新已有的节点，还是新节点初次生成配置。

```
$ cloud-config update-node --domain node0

$ cloud-config update-node --domain node1

$ tree test-chain-node*
test-chain-node0
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-node1
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
```

#### update-yaml

参数：

```
        --chain-name <CHAIN_NAME>
            set chain name [default: test-chain]

        --config-dir <CONFIG_DIR>
            set config file directory, default means current directory [default: .]

        --config-name <CONFIG_NAME>
            set config file name [default: config.toml]

        --docker-registry <DOCKER_REGISTRY>
            docker registry [default: docker.io]

        --docker-repo <DOCKER_REPO>
            docker repo [default: citacloud]

        --domain <DOMAIN>
            domain of node

        --enable-debug
            is enable debug

        --limits-cpu <LIMITS_CPU>
            container resource requirements -- limits cpu [default: 4000m]

        --limits-memory <LIMITS_MEMORY>
            container resource requirements -- limits memory [default: 8192Mi]

        --pull-policy <PULL_POLICY>
            image pull policy: IfNotPresent or Always [default: IfNotPresent]

        --requests-cpu <REQUESTS_CPU>
            container resource requirements -- requests cpu [default: 10m]

        --requests-memory <REQUESTS_MEMORY>
            container resource requirements -- requests memory [default: 32Mi]

        --storage-capacity <STORAGE_CAPACITY>
            storage capacity [default: 10Gi]

        --storage-class <STORAGE_CLASS>
            storage class

```

说明：
1. `domain`为必选参数，作为节点的标识，表示要操作的节点。
2. `storage-class`为必选参数，指定节点在`k8s`集群中的持久化存储使用的存储类。
3. `limits-cpu`,`limits-memory`,`requests-cpu`,`requests-memory`用于设定微服务的硬件资源需求。请根据实际运行环境的硬件配置进行调整，以获得最佳性能体验。

```
$ cloud-config update-yaml --domain node0 --storage-class nfs-client
$ ls test-chain-node0/yamls
test-chain-node0-cm-account.yaml  test-chain-node0-cm-config.yaml  test-chain-node0-cm-log.yaml  test-chain-node0-svc.yaml  test-chain-node0.yaml
```

#### delete-chain

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
```

说明：
1. 该命令会删除所有跟指定链相关的文件夹及文件，`使用时要慎重`。

### 开发环境

约定：
1. 节点的`ip`都为`localhost`。
2. 节点的网络端口从`40000`开始往后顺延。
3. 节点的`domain`使用节点的序号。
4. 节点的`kms`密码都为`123456`。
5. 节点各个微服务的`gRPC`端口从`50000 + i*1000`开始往后顺延。其中`i`为节点的序号。
6. 日志默认输出到文件。
7. 增加节点只能在最大的节点序号往后增加。
8. 删除节点也只能从最大的节点序号开始往前删除。

适用于开发阶段，在单机非容器环境直接跑链进行测试。

#### create-dev

参数：
```
$ cloud-config create-dev -h
cloud-config-create-dev 

create config in env dev

USAGE:
    cloud-config create-dev [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Print help information
        --is-bft     is consensus bft
        --is-tls     is network tls
        --is-eth     is kms eth
    -V, --version    Print version information

OPTIONS:
        --chain-name <CHAIN_NAME>      set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>      set config file directory, default means current directory
                                       [default: .]
        --log-level <LOG_LEVEL>        log level [default: info]
        --peers-count <PEERS_COUNT>    set initial node number [default: 4]
```

说明：
1. `--is-tls`标识网络微服务是否选择了`network_tls`。
2. `--is-bft`标识共识微服务是否选择了`consensus_bft`。
3. `--is-eth`标识kms微服务是否选择了`kms_eth`。

```
$ cloud-config create-dev  
key_id:1, address:5928e5527c4c1c6ba75059c6aa2d5832cd543eab
key_id:1, address:c83e8a0aa624f26d8344badfe6e0e9149e50965a
key_id:1, address:74866e7bf097f3889465ceab1a0028cca65495f2
key_id:1, address:e4a4f3f10cb98969b1809a8bc99f0f34d86f788e
key_id:1, address:c36d6b36b049a9d7ce30c472be389f6ff45f26ef

$ tree test-chain*
test-chain
├── accounts
│   ├── 5928e5527c4c1c6ba75059c6aa2d5832cd543eab
│   │   ├── key_id
│   │   └── kms.db
│   ├── 74866e7bf097f3889465ceab1a0028cca65495f2
│   │   ├── key_id
│   │   └── kms.db
│   ├── c36d6b36b049a9d7ce30c472be389f6ff45f26ef
│   │   ├── key_id
│   │   └── kms.db
│   ├── c83e8a0aa624f26d8344badfe6e0e9149e50965a
│   │   ├── key_id
│   │   └── kms.db
│   └── e4a4f3f10cb98969b1809a8bc99f0f34d86f788e
│       ├── key_id
│       └── kms.db
├── ca_cert
├── certs
└── chain_config.toml
test-chain-0
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-1
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-2
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-3
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
```

#### append-dev

参数：
```
$ cloud-config append-dev -h
cloud-config-append-dev 

append node in env dev

USAGE:
    cloud-config append-dev [OPTIONS]

FLAGS:
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --log-level <LOG_LEVEL>      log level [default: info]
```

```
$ cloud-config append-dev
key_id:1, address:50887f9dc812402e41fab3129c955c93dc176896

$ tree test-chain*
test-chain
├── accounts
│   ├── 50887f9dc812402e41fab3129c955c93dc176896
│   │   ├── key_id
│   │   └── kms.db
│   ├── 5928e5527c4c1c6ba75059c6aa2d5832cd543eab
│   │   ├── key_id
│   │   └── kms.db
│   ├── 74866e7bf097f3889465ceab1a0028cca65495f2
│   │   ├── key_id
│   │   └── kms.db
│   ├── c36d6b36b049a9d7ce30c472be389f6ff45f26ef
│   │   ├── key_id
│   │   └── kms.db
│   ├── c83e8a0aa624f26d8344badfe6e0e9149e50965a
│   │   ├── key_id
│   │   └── kms.db
│   └── e4a4f3f10cb98969b1809a8bc99f0f34d86f788e
│       ├── key_id
│       └── kms.db
├── ca_cert
├── certs
└── chain_config.toml
test-chain-0
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-1
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-2
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-3
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-4
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
```

#### delete-dev

参数：
```
$ cloud-config delete-dev -h
cloud-config-delete-dev 

delete node in env dev

USAGE:
    cloud-config delete-dev [OPTIONS]

FLAGS:
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --log-level <LOG_LEVEL>      log level [default: info]

```

```
$ cloud-config delete-dev

$ tree test-chain*
test-chain
├── accounts
│   ├── 5928e5527c4c1c6ba75059c6aa2d5832cd543eab
│   │   ├── key_id
│   │   └── kms.db
│   ├── 74866e7bf097f3889465ceab1a0028cca65495f2
│   │   ├── key_id
│   │   └── kms.db
│   ├── c36d6b36b049a9d7ce30c472be389f6ff45f26ef
│   │   ├── key_id
│   │   └── kms.db
│   ├── c83e8a0aa624f26d8344badfe6e0e9149e50965a
│   │   ├── key_id
│   │   └── kms.db
│   └── e4a4f3f10cb98969b1809a8bc99f0f34d86f788e
│       ├── key_id
│       └── kms.db
├── ca_cert
├── certs
└── chain_config.toml
test-chain-0
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-1
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-2
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-3
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
```


### k8s环境

约定：
1. 超级管理员账户由用户自行创建并通过参数传入。
2. 节点的`kms`密码都由用户设置并通过参数传入。
3. 节点各个微服务的`gRPC`端口固定从`50000`开始往后顺延。
4. 节点的网络监听端口固定为`40000`。
5. 日志默认输出到`stdout`。

适用于演示或者生产阶段，在k8s环境部署链。

#### create-k8s

参数：
```
        --admin <ADMIN>
            set admin

        --block_interval <BLOCK_INTERVAL>
            set system config block_interval [default: 3]

        --block_limit <BLOCK_LIMIT>
            set system config block_limit [default: 100]

        --chain-name <CHAIN_NAME>
            set chain name [default: test-chain]

        --chain_id <CHAIN_ID>
            set system config chain_id [default: ]

        --config-dir <CONFIG_DIR>
            set config file directory, default means current directory [default: .]

        --consensus_image <CONSENSUS_IMAGE>
            set consensus micro service image name (consensus_bft/consensus_raft) [default:
            consensus_raft]

        --consensus_tag <CONSENSUS_TAG>
            set consensus micro service image tag [default: latest]

        --controller_image <CONTROLLER_IMAGE>
            set controller micro service image name (controller) [default: controller]

        --controller_tag <CONTROLLER_TAG>
            set controller micro service image tag [default: latest]

        --executor_image <EXECUTOR_IMAGE>
            set executor micro service image name (executor_evm) [default: executor_evm]

        --executor_tag <EXECUTOR_TAG>
            set executor micro service image tag [default: latest]

        --kms-password-list <KMS_PASSWORD_LIST>
            kms db password list, splited by ,

        --kms_image <KMS_IMAGE>
            set kms micro service image name (kms_eth/kms_sm) [default: kms_sm]

        --kms_tag <KMS_TAG>
            set kms micro service image tag [default: latest]

        --log-level <LOG_LEVEL>
            log level [default: info]

        --network_image <NETWORK_IMAGE>
            set network micro service image name (network_tls/network_p2p) [default: network_p2p]

        --network_tag <NETWORK_TAG>
            set network micro service image tag [default: latest]

        --nodelist <NODE_LIST>
            node list looks like localhost:40000:node0,localhost:40001:node1

        --prevhash <PREVHASH>
            set genesis prevhash [default:
            0x0000000000000000000000000000000000000000000000000000000000000000]

        --storage_image <STORAGE_IMAGE>
            set storage micro service image name (storage_rocksdb) [default: storage_rocksdb]

        --storage_tag <STORAGE_TAG>
            set storage micro service image tag [default: latest]

        --timestamp <TIMESTAMP>
            set genesis timestamp [default: 0]

        --version <VERSION>
            set system config version [default: 0]
```

说明:
1. `admin`为必选参数。使用用户事先创建好的超级管理员账户地址。
2. `kms-password-list`为必选参数。用逗号隔开的多个节点的`kms`数据库密码。
3. `nodelist`为必选参数。值为多个节点的网络地址,用逗号分隔。每个节点的网络地址包含`ip`,`port`和`domain`，之间用分号分隔。
4. `kms-password-list`和`nodelist`的数量必须相同。

```
$ cloud-config create-k8s --admin 8f81961f263f45f88230375623394c9301c033e7 --kms-password-list 123456,123456 --nodelist localhost:40000:node0,localhost:40001:node1
key_id:1, address:4b209a87a31f67f2c64a7583301ff5a360796241
key_id:1, address:b7f1a7d398da3eee28a5f01f02e8ae05317270c8

$ tree test-chain*
test-chain
├── accounts
│   ├── 4b209a87a31f67f2c64a7583301ff5a360796241
│   │   ├── key_id
│   │   └── kms.db
│   └── b7f1a7d398da3eee28a5f01f02e8ae05317270c8
│       ├── key_id
│       └── kms.db
├── ca_cert
├── certs
└── chain_config.toml
test-chain-node0
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-node1
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
```

#### append-k8s

参数：
```
        --chain-name <CHAIN_NAME>
            set chain name [default: test-chain]

        --config-dir <CONFIG_DIR>
            set config file directory, default means current directory [default: .]

        --kms-password <KMS_PASSWORD>
            kms db password

        --log-level <LOG_LEVEL>
            log level [default: info]

        --node <NODE>
            node network address looks like localhost:40002:node2
```

说明：
1. `kms-password`为必选参数。值为新增节点的`kms`数据库密码。
2. `node`为必选参数。值为新增节点的网络地址包含`ip`,`port`和`domain`，之间用分号分隔。


```
$ cloud-config append-k8s --kms-password 123456 --node localhost:40002:node2
key_id:1, address:6ee268553c09f203dedd54f7d0a678c8a9439915

$ tree test-chain*
test-chain
├── accounts
│   ├── 4b209a87a31f67f2c64a7583301ff5a360796241
│   │   ├── key_id
│   │   └── kms.db
│   ├── 6ee268553c09f203dedd54f7d0a678c8a9439915
│   │   ├── key_id
│   │   └── kms.db
│   └── b7f1a7d398da3eee28a5f01f02e8ae05317270c8
│       ├── key_id
│       └── kms.db
├── ca_cert
├── certs
└── chain_config.toml
test-chain-node0
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-node1
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-node2
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
```

#### delete-k8s

参数：
```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --domain <DOMAIN>            domain of node that want to delete
```

说明：
1. `domain`为必选参数。值为要删除节点的标识，与创建链和增加节点时的`domain`保持一致。

```
$ cloud-config delete-k8s --domain node2

$ tree test-chain*
test-chain
├── accounts
│   ├── 4b209a87a31f67f2c64a7583301ff5a360796241
│   │   ├── key_id
│   │   └── kms.db
│   └── b7f1a7d398da3eee28a5f01f02e8ae05317270c8
│       ├── key_id
│       └── kms.db
├── ca_cert
├── certs
└── chain_config.toml
test-chain-node0
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
test-chain-node1
├── config.toml
├── controller-log4rs.yaml
├── executor-log4rs.yaml
├── kms.db
├── kms-log4rs.yaml
├── network-log4rs.yaml
├── node_config.toml
└── storage-log4rs.yaml
```

#### migrate
从v6.1.0的链配置中读取数据，生成v6.3.0链的配置。生成的配置中不包含运行时产生的数据。

```
    -c, --consensus-type <CONSENSUS_TYPE>
            Consensus type, only `raft` or `bft` is supported [default: raft]

    -d, --chain-dir <CHAIN_DIR>
            The old chain dir

    -k, --kms-password-list <KMS_PASSWORD_LIST>
            KMS password list, e.g. "node1password,node2password"

    -l, --nodelist <NODE_LIST>
            Node list, e.g. "127.0.0.1:40000,citacloud.org:40001"

    -n, --chain-name <CHAIN_NAME>
            Name of the chain

    -o, --out-dir <OUT_DIR>
            The output dir for the upgraded chain
```

示例

```
$ tree old-chain

old-chain
├── test-chain
│
├── test-chain-0
│   ├── chain_data
|   |       (omitted)
│   ├── consensus-config.toml
│   ├── consensus-log4rs.yaml
│   ├── controller-config.toml
│   ├── controller-log4rs.yaml
│   ├── data
|   |       (omitted)
│   ├── executor-log4rs.yaml
│   ├── genesis.toml
│   ├── init_sys_config.toml
│   ├── key_file
│   ├── key_id
│   ├── kms-log4rs.yaml
│   ├── logs
|   |       (omitted)
│   ├── network-config.toml
│   ├── network_key
│   ├── network-log4rs.yaml
│   ├── node_address
│   ├── node_key
│   ├── raft-data-dir
|   |       (omitted)
│   └── storage-log4rs.yaml
|── test-chain-1
|       (omitted)
|── test-chain-2
|       (omitted)
└── test-chain-3
        (omitted)

$ cloud-config migrate \
    --chain-dir test-chain \
    --chain-name test-chain \
    --out-dir new-test-chain \
    --consensus-type raft \
    --kms-password-list 123456,123456,123456,123456 \
    --nodelist 127.0.0.1:40000,127.0.0.1:40001,127.0.0.1:40002,127.0.0.1:40003

$ tree new-test-chain
new-test-chain/
├── test-chain
│   ├── accounts
│   │   ├── 1dc41ae9cd4eeebf12a053e0abba49a909530981
│   │   │   ├── key_id
│   │   │   └── kms.db
│   │   ├── 287d70c1e4263ff38ad993cb2c18d01e3d5a151b
│   │   │   ├── key_id
│   │   │   └── kms.db
│   │   ├── 2ac2df73fff75e5260a75f2c0df8c06b08aff09f
│   │   │   ├── key_id
│   │   │   └── kms.db
│   │   └── 83b097e4f547bb49c51dc17b8821e6b0a91b31da
│   │       ├── key_id
│   │       └── kms.db
│   ├── ca_cert
│   │   ├── cert.pem
│   │   └── key.pem
│   ├── certs
│   │   ├── 0
│   │   │   ├── cert.pem
│   │   │   ├── csr.pem
│   │   │   └── key.pem
│   │   ├── 1
│   │   │   ├── cert.pem
│   │   │   ├── csr.pem
│   │   │   └── key.pem
│   │   ├── 2
│   │   │   ├── cert.pem
│   │   │   ├── csr.pem
│   │   │   └── key.pem
│   │   └── 3
│   │       ├── cert.pem
│   │       ├── csr.pem
│   │       └── key.pem
│   └── chain_config.toml
├── test-chain-0
│   ├── config.toml
│   ├── controller-log4rs.yaml
│   ├── executor-log4rs.yaml
│   ├── kms.db
│   ├── kms-log4rs.yaml
│   ├── network-log4rs.yaml
│   ├── node_config.toml
│   └── storage-log4rs.yaml
├── test-chain-1
|       (omitted)
├── test-chain-2
|       (omitted)
└── test-chain-3
        (omitted)
```
