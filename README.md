# cloud-config

创建链的配置文件和部署文件。

### 依赖

* rust: 1.59+

### 安装

```
cargo install --path .
```

### 用法

```
$ cloud-config -h
cloud-config 6.6.4
Rivtower Technologies <contact@rivtower.com>

Usage: cloud-config <COMMAND>

Commands:
  init-chain         init a chain
  init-chain-config  init chain config
  set-admin          set admin of chain
  set-validators     set validators of chain
  append-validator   append a validator into chain
  set-nodelist       set node list
  append-node        append a node into chain
  delete-node        delete a node from chain
  init-node          init node
  update-node        update node
  delete-chain       delete a chain
  new-account        new account
  import-account     import account
  create-ca          create CA
  create-csr         create csr
  sign-csr           sign csr
  create-dev         create config in env dev
  append-dev         append node in env dev
  delete-dev         delete node in env dev
  create-k8s         create config in env k8s
  append-k8s         append node in env k8s
  delete-k8s         delete node in env k8s
  set-stage          set stage
  import-ca          import ca
  import-cert        import node cert
  update-yaml        update k8s yaml
  delete-validator   delete a validator from chain
  help               Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version informatio
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
4. [new-account](/src/new_account.rs)创建账户。会在`$(config-dir)/$(chain-name)/accounts/`下，创建以账户地址为名的文件夹，里面有`node_address`，`private_key`，和`validator_address`三个文件。
5. [import-account](/src/import_account.rs)导入私钥的方式创建账户。会在`$(config-dir)/$(chain-name)/accounts/`下，创建以账户地址为名的文件夹，里面有`node_address`，`private_key`，和`validator_address`三个文件。
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
3. [set-admin](/src/set_admin.rs)。设置管理员账户。账户需要事先通过[new-account](/src/new_account.rs)子命令创建。默认网络微服务为`network_zenoh`，还需要通过[create-ca](/src/create_ca.rs)创建链的根证书。
4. [set-validators](/src/set_validators.rs)。设置共识节点账户列表。账户同样需要事先通过[new-account](/src/new_account.rs)子命令，由各个共识节点分别创建，然后将账户地址集中到一起进行设置。
5. [set-nodelist](/src/set_nodelist.rs)。设置节点网络地址列表。各个节点参与方需要根据自己的网络环境，预先保留节点的`ip`，`port`和`domain`。然后将相关信息集中到一起进行设置。至此，`链级配置`信息设置完成，可以下发配置文件`chain_config.toml`到各个节点。默认网络微服务为`network_zenoh`，则需要通过`create-csr`根据节点的`domain`为各个节点创建证书和签名请求。然后请求`CA`通过`sign-crs`处理签名请求，并下发生成的`cert.pem`到各个节点。
6. [set-stage](/src/set_stage.rs)设置链配置进行到的阶段。将链的配置过程分为三个阶段，分别为`init`，`public`和`finalize`，一些子命令会根据当前所处的阶段，来判断是否可以进行操作，以避免一些误操作。`init-chain-config`初始化链的配置时，默认设置阶段为`init`，该阶段可以重复进行`init-chain-config`以修改链的基础配置；`set-admin`只能在`init`阶段执行，并且会自动将阶段修改为`public`；`append-validator`，`delete-validator`和`set-validators`只能在`public`阶段进行；此后需要手工执行该命令将阶段修改为`finalize`，此时链的配置完成，不能再修改除`节点网络地址列表`之外的配置，并且此后才可以进行`init-node`操作。辅助命令和`delete-chain`不受阶段的限制。
7. [init-node](/src/init_node.rs)。设置`节点配置`信息。这步操作由各个节点的参与方独立设置，节点之间可以不同。执行之后会生成`$(config-dir)/$(chain-name)-$(domain)/node_config.toml`
8. [update-node](/src/update_node.rs)。根据之前设置的`链级配置`和`节点配置`，生成每个节点所需的微服务配置文件。
9. [update-yaml](/src/update_yaml.rs)。根据之前设置的`链级配置`和`节点配置`，生成每个节点部署到`k8s`环境所需的资源文件。
10. [delete-chain](/src/delete_chain.rs)删除链。删除属于该链的所有文件夹以及其中的文件，`使用时要慎重`。

此外还有一些其他操作的子命令。

比如增加单个共识节点的`append-validator`，删除单个共识节点的`delete-validator`；增加单个网络节点的`append-node`，删除单个网络节点的`delete-node`。

在前述流程的基础上，封装了更高级更方便使用的子命令。

比如针对开发环境，有`create-dev`，`append-dev`和`delete-dev`，使用开发环境约定好的默认值，无需执行这么多子命令，也无需传递大量参数就可以生成和操作一条链的配置。

针对演示或者生产阶段的`k8s`环境，有`create-k8s`，`append-k8s`和`delete-k8s`。


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
$ tree -a test-chain/
test-chain/
├── .gitignore
├── accounts
│   └── .gitkeep
├── ca_cert
│   └── .gitkeep
└── certs
    └── .gitkeep
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
            set consensus micro service image name (consensus_raft/consensus_overlord)
            [default: consensus_overlord]

        --consensus_tag <CONSENSUS_TAG>
            set consensus micro service image tag [default: latest]

        --controller_image <CONTROLLER_IMAGE>
            set controller micro service image name (controller) [default: controller]

        --controller_tag <CONTROLLER_TAG>
            set controller micro service image tag [default: latest]

        --crypto_image <CRYPTO_IMAGE>
            set crypto micro service image name (crypto_eth/crypto_sm) [default: crypto_sm]

        --crypto_tag <CRYPTO_TAG>
            set crypto micro service image tag [default: latest]

        --executor_image <EXECUTOR_IMAGE>
            set executor micro service image name (executor_evm) [default: executor_evm]

        --executor_tag <EXECUTOR_TAG>
            set executor micro service image tag [default: latest]

        --network_image <NETWORK_IMAGE>
            set network micro service image name (network_zenoh) [default: network_zenoh]

        --network_tag <NETWORK_TAG>
            set network micro service image tag [default: latest]

        --prevhash <PREVHASH>
            set genesis prevhash [default:
            0x0000000000000000000000000000000000000000000000000000000000000000]

        --quota-limit <QUOTA_LIMIT>
            set one block contains tx limit, default 1073741824 [default: 1073741824]

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

$ tree test-chain/
test-chain/
├── accounts
├── ca_cert
├── certs
└── chain_config.toml

$ cat test-chain/chain_config.toml
node_network_address_list = []
stage = 'Init'

[[micro_service_list]]
image = 'network_zenoh'
tag = 'latest'

[[micro_service_list]]
image = 'consensus_overlord'
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
image = 'crypto_sm'
tag = 'latest'

[genesis_block]
prevhash = '0x0000000000000000000000000000000000000000000000000000000000000000'
timestamp = 1649418157966

[system_config]
admin = ''
block_interval = 3
block_limit = 100
chain_id = '63586a3c0255f337c77a777ff54f0040b8c388da04f23ecee6bfd4953a6512b4'
quota_limit = 1073741824
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
```

```
$ cloud-config new-account   
node_address: aeaa6e333b8ed911f89acd01e88e3d9892da87b5 validator_address: aeaa6e333b8ed911f89acd01e88e3d9892da87b5

$ cloud-config new-account
node_address: 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa validator_address: 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa

$ cloud-config new-account
node_address: 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd validator_address: 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd

$ tree test-chain 
test-chain
├── accounts
│   ├── 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa
│   │   ├── node_address
│   │   ├── private_key
│   │   └── validator_address
│   ├── 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd
│   │   ├── node_address
│   │   ├── private_key
│   │   └── validator_address
│   └── aeaa6e333b8ed911f89acd01e88e3d9892da87b5
│   │   ├── node_address
│   │   ├── private_key
│   │   └── validator_address
├── ca_cert
├── certs
└── chain_config.toml
```

说明：
1. 这里会创建两个地址，一个用于标识节点的`node_address`，一个用于共识`validator`的`validator_address`。
2. 默认情况下两个地址是一样的。但是当共识微服务选择`consensus_overlord`时两者不一样，注意区分。


#### import-account

参数：

```
--chain-name <CHAIN_NAME>
    set chain name [default: test-chain]

--config-dir <CONFIG_DIR>
    set config file directory, default means current directory [default: .]

--privkey <PRIVKEY>
    hex encoded private key
```

```
$ cloud-config import-account --privkey 60e7b47ee260516dbfedf8e80ff38830bae8663cc498a900a610c147cae94344
node_address: 097913007f2c8d9ac87f89664fc70977fee6bf9a validator_address: 097913007f2c8d9ac87f89664fc70977fee6bf9a

$ tree test-chain/
test-chain/
├── accounts
│   └── 097913007f2c8d9ac87f89664fc70977fee6bf9a
│   │   ├── node_address
│       ├── private_key
│       └── validator_address
├── ca_cert
├── certs
└── chain_config.toml
```

说明：

1. 这里会创建两个地址，一个用于标识节点的`node_address`，一个用于共识`validator`的`validator_address`。
2. 默认情况下两个地址是一样的。但是当共识微服务选择`consensus_overlord`时两者不一样，注意区分。

#### set-admin

参数：

```
        --admin <ADMIN>              set admin
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
```

说明：

1. `admin`为必选参数。值为之前用`new-account`创建的`node_address`地址。


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

1. `validators`为必选参数。值为多个之前用`new-account`创建的`validator_address`地址,用逗号分隔。

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

1. `validator`为必选参数。值为之前用`new-account`创建的`validator_address`地址。
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
                                     localhost:40000:node0:k8s_cluster1,localhost:40001:node1:k8s_cluster2
                                     last slice is optional, none means not k8s env
```

说明：

1. `nodelist`为必选参数。值为多个节点的网络地址,用逗号分隔。每个节点的网络地址包含`ip`,`port`，`domain`，`cluster name`，之间用分号分隔。
2. `cluster name`是节点所在的`k8s`集群的名称。出于兼容性考虑，非`k8s`环境该项可以省略，会自动生成一个随机的集群名称。
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
2. `cluster name`是节点所在的`k8s`集群的名称。出于兼容性考虑，非`k8s`环境该项可以省略，会自动生成一个随机的集群名称。
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
├── ca_cert
│   ├── cert.pem
│   └── key.pem
├── certs
└── chain_config.toml
```

说明：
1. 该命令生成文件形式的根证书，存放在`ca_cert`目录下。

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
2. 证书中真正使用的`domain`是`$(chain-name)-$(domain)`。

```
$ cloud-config create-csr --domain node0 

$ cloud-config create-csr --domain node1

$ tree test-chain
test-chain
├── accounts
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

#### init-node

参数：

```
      --chain-name <CHAIN_NAME>
          set chain name [default: test-chain]
      --config-dir <CONFIG_DIR>
          set config file directory, default means current directory [default: .]
      --domain <DOMAIN>
          domain of node
      --network-port <NETWORK_PORT>
          grpc network_port of node [default: 50000]
      --consensus-port <CONSENSUS_PORT>
          grpc consensus_port of node [default: 50001]
      --executor-port <EXECUTOR_PORT>
          grpc executor_port of node [default: 50002]
      --storage-port <STORAGE_PORT>
          grpc storage_port of node [default: 50003]
      --controller-port <CONTROLLER_PORT>
          grpc controller_port of node [default: 50004]
      --crypto-port <CRYPTO_PORT>
          grpc crypto_port of node [default: 50005]
      --network-listen-port <NETWORK_LISTEN_PORT>
          network listen port of node [default: 40000]
      --log-level <LOG_LEVEL>
          log level [default: info]
      --log-file-path <LOG_FILE_PATH>
          log file path
      --jaeger-agent-endpoint <JAEGER_AGENT_ENDPOINT>
          jaeger agent endpoint
      --account <ACCOUNT>
          account of node
      --network-metrics-port <NETWORK_METRICS_PORT>
          network metrics port of node [default: 60000]
      --consensus-metrics-port <CONSENSUS_METRICS_PORT>
          consensus metrics port of node [default: 60001]
      --executor-metrics-port <EXECUTOR_METRICS_PORT>
          executor metrics port of node [default: 60002]
      --storage-metrics-port <STORAGE_METRICS_PORT>
          storage metrics port of node [default: 60003]
      --controller-metrics-port <CONTROLLER_METRICS_PORT>
          controller metrics port of node [default: 60004]
      --crypto-metrics-port <CRYPTO_METRICS_PORT>
          crypto metrics port of node [default: 60005]
      --disable-metrics
          disable metrics
      --is-danger
          is chain in danger mode
```

说明：
1. 参数部分基本对应`节点配置`数据结构，具体含义参见设计部分的描述。
2. `domain`为必选参数，作为节点的标识，节点文件夹将会以`$(chanin-name)-$(domain)`的形式命名。
3. `account`为必选参数，表示该节点要使用的账户地址。值为之前用`new-account`创建的`node_address`地址。

```
$ cloud-config init-node --domain node0 --account 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa
 
$ cloud-config init-node --domain node1 --account 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd

$ tree test-chain-node* 
test-chain-node0
├── accounts
├── ca_cert
├── certs
├── chain_config.toml
└── node_config.toml
test-chain-node1
├── accounts
├── ca_cert
├── certs
├── chain_config.toml
└── node_config.toml


$ cat test-chain-node0/node_config.toml
account = '5274d0171fd28a76d98f1e743298adf0c9b7d771'
log_level = 'info'
network_listen_port = 40000

[grpc_ports]
consensus_port = 50001
controller_port = 50004
crypto_port = 50005
executor_port = 50002
network_port = 50000
storage_port = 50003

[metrics_ports]
consensus_metrics_port = 60001
controller_metrics_port = 60004
crypto_metrics_port = 60005
executor_metrics_port = 60002
network_metrics_port = 60000
storage_metrics_port = 60003
```

#### update-node

参数：

```
        --chain-name <CHAIN_NAME>      set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>      set config file directory, default means current directory
                                       [default: .]
        --config-name <CONFIG_NAME>    set config file name [default: config.toml]
        --domain <DOMAIN>              domain of node
        --is-dev                       is for dev env
        --no-stdout                    disable output to stdout
```

说明：
1. `domain`为必选参数，作为节点的标识，表示要操作的节点。
2. `is-dev`用于区别是`dev`环境，亦或`k8s`环境。

```
$ cloud-config update-node --domain node0

$ cloud-config update-node --domain node1

$ tree test-chain-node*
test-chain-node0
├── chain_config.toml
├── config.toml
├── node_address
├── node_config.toml
├── private_key
└── validator_address
test-chain-node1
├── chain_config.toml
├── config.toml
├── node_address
├── node_config.toml
├── private_key
└── validator_address
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

        --disable-health-check
            is disable health-check

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
cm-account.yaml  cm-config.yaml  cm-log.yaml  node-svc.yaml  statefulset.yaml
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
4. 节点各个微服务的`gRPC`端口从`50000 + i*1000`开始往后顺延。其中`i`为节点的序号。
4. 节点各个微服务的`metrics exporter`端口从`60000 + i*100`开始往后顺延。其中`i`为节点的序号。
5. 增加节点只能在最大的节点序号往后增加。
6. 删除节点也只能从最大的节点序号开始往前删除。

适用于开发阶段，在单机非容器环境直接跑链进行测试。

#### create-dev

参数：
```
$ cloud-config create-dev -h
create config in env dev

Usage: cloud-config create-dev [OPTIONS]

Options:
      --chain-name <CHAIN_NAME>
          set chain name [default: test-chain]
      --config-dir <CONFIG_DIR>
          set config file directory, default means current directory [default: .]
      --peers-count <PEERS_COUNT>
          set initial node number [default: 4]
      --log-level <LOG_LEVEL>
          log level [default: info]
      --log-file-path <LOG_FILE_PATH>
          log file path [default: ./logs]
      --jaeger-agent-endpoint <JAEGER_AGENT_ENDPOINT>
          jaeger agent endpoint
      --is-raft
          is consensus raft
      --is-eth
          is crypto eth
      --is-danger
          is chain in danger mode
      --disable-metrics
          disable metrics
  -h, --help
          Print help
```

说明：
1. `--is-raft`标识`consensus`微服务是否选择了`consensus_raft`。
2. `--is-eth`标识`crypto`微服务是否选择了`crypto_eth`。

```
$ cloud-config create-dev
node_address: fca6ddc659665f94294bb73ef79c0f825b30ffed validator_address: fca6ddc659665f94294bb73ef79c0f825b30ffed
node_address: 4e85201492995fb4d5995540165bc1af7e4df958 validator_address: 4e85201492995fb4d5995540165bc1af7e4df958
node_address: f7e55b05b93399b0c61f71063123bc6ecfa62b48 validator_address: f7e55b05b93399b0c61f71063123bc6ecfa62b48
node_address: 856c583be3b82e171feba44a991319e9a989deed validator_address: 856c583be3b82e171feba44a991319e9a989deed
node_address: 804f620d1cb955f10381501da12e985fd76ab96d validator_address: 804f620d1cb955f10381501da12e985fd76ab96d

$ ls test-chain-*
test-chain-0:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-1:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-2:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-3:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address
```

#### append-dev

参数：
```
$ cloud-config append-dev -h
append node in env dev

Usage: cloud-config append-dev [OPTIONS]

Options:
      --chain-name <CHAIN_NAME>
          set chain name [default: test-chain]
      --config-dir <CONFIG_DIR>
          set config file directory, default means current directory [default: .]
      --log-level <LOG_LEVEL>
          log level [default: info]
      --log-file-path <LOG_FILE_PATH>
          log file path
      --jaeger-agent-endpoint <JAEGER_AGENT_ENDPOINT>
          jaeger agent endpoint
      --is-danger
          is chain in danger mode
      --disable-metrics
          disable metrics
  -h, --help
          Print help
```

```
$ cloud-config append-dev
node_address: 48eb184fe084387a6d03d78c8b5cd2794a58de5e validator_address: 48eb184fe084387a6d03d78c8b5cd2794a58de5e

$ ls test-chain-*
test-chain-0:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-1:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-2:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-3:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-4:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address
```

#### delete-dev

参数：
```
$ cloud-config delete-dev -h
cloud-config-delete-dev
delete node in env dev

USAGE:
    cloud-config delete-dev [OPTIONS]

OPTIONS:
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
    -h, --help                       Print help information
```

```
$ cloud-config delete-dev
$ ls test-chain-*
test-chain-0:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-1:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-2:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-3:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address
```


### k8s环境

约定：
1. 超级管理员账户由用户自行创建并通过参数传入。
2. 节点各个微服务的`gRPC`端口固定从`50000`开始依次分配给各微服务。
2. 节点各个微服务的`metrics exporter`端口固定从`60000`依次分配给各微服务。
3. 节点的网络监听端口固定为`40000`。

适用于演示或者生产阶段，在k8s环境部署链。

#### create-k8s

参数：

```
      --chain-name <CHAIN_NAME>
          set chain name [default: test-chain]
      --config-dir <CONFIG_DIR>
          set config file directory, default means current directory [default: .]
      --timestamp <TIMESTAMP>
          set genesis timestamp [default: 0]
      --prevhash <PREVHASH>
          set genesis prevhash [default: 0x0000000000000000000000000000000000000000000000000000000000000000]
      --version <VERSION>
          set system config version [default: 0]
      --chain_id <CHAIN_ID>
          set system config chain_id [default: ]
      --block_interval <BLOCK_INTERVAL>
          set system config block_interval [default: 3]
      --block_limit <BLOCK_LIMIT>
          set system config block_limit [default: 100]
      --quota-limit <QUOTA_LIMIT>
          set one block contains quota limit, default 1073741824 [default: 1073741824]
      --network_image <NETWORK_IMAGE>
          set network micro service image name (network_zenoh) [default: network_zenoh]
      --network_tag <NETWORK_TAG>
          set network micro service image tag [default: latest]
      --consensus_image <CONSENSUS_IMAGE>
          set consensus micro service image name (consensus_raft/consensus_overlord) [default: consensus_overlord]
      --consensus_tag <CONSENSUS_TAG>
          set consensus micro service image tag [default: latest]
      --executor_image <EXECUTOR_IMAGE>
          set executor micro service image name (executor_evm) [default: executor_evm]
      --executor_tag <EXECUTOR_TAG>
          set executor micro service image tag [default: latest]
      --storage_image <STORAGE_IMAGE>
          set storage micro service image name (storage_rocksdb) [default: storage_rocksdb]
      --storage_tag <STORAGE_TAG>
          set storage micro service image tag [default: latest]
      --controller_image <CONTROLLER_IMAGE>
          set controller micro service image name (controller) [default: controller]
      --controller_tag <CONTROLLER_TAG>
          set controller micro service image tag [default: latest]
      --crypto_image <CRYPTO_IMAGE>
          set crypto micro service image name (crypto_eth/crypto_sm) [default: crypto_sm]
      --crypto_tag <CRYPTO_TAG>
          set crypto micro service image tag [default: latest]
      --admin <ADMIN>
          set admin
      --nodelist <NODE_LIST>
          node list looks like localhost:40000:node0:k8s_cluster1,localhost:40001:node1:k8s_cluster2 last slice is optional, none means not k8s env
      --log-level <LOG_LEVEL>
          log level [default: info]
      --log-file-path <LOG_FILE_PATH>
          log file path
      --jaeger-agent-endpoint <JAEGER_AGENT_ENDPOINT>
          jaeger agent endpoint
      --is-danger
          is chain in danger mode
      --disable-metrics
          disable metrics
```

说明:
1. `admin`为必选参数。使用用户事先创建好的超级管理员账户地址。
2. `nodelist`为必选参数。值为多个节点的网络地址,用逗号分隔。每个节点的网络地址包含`ip`,`port`，`domain`，`cluster name`，之间用分号分隔。`cluster name`是节点所在的`k8s`集群的名称。出于兼容性考虑，非`k8s`环境该项可以省略，会自动生成一个随机的集群名称。

```
$ ./target/debug/cloud-config create-k8s --admin 0xff8456931c10a9b02ec4a657ee05e724ecad9372 --nodelist localhost:40000:node0:k8s,localhost:40001:node1:k8s
node_address: 0467a471d83343ce99888180ca8cabe0e49a0ae3 validator_address: 0467a471d83343ce99888180ca8cabe0e49a0ae3
node_address: 24bec821e0ba3ea1e8d35c60a6debb57daa9fa41 validator_address: 24bec821e0ba3ea1e8d35c60a6debb57daa9fa41

$ ls test-chain-node*
test-chain-node0:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-node1:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address
```

#### append-k8s

参数：
```
      --chain-name <CHAIN_NAME>
          set chain name [default: test-chain]
      --config-dir <CONFIG_DIR>
          set config file directory, default means current directory [default: .]
      --log-level <LOG_LEVEL>
          log level [default: info]
      --log-file-path <LOG_FILE_PATH>
          log file path
      --jaeger-agent-endpoint <JAEGER_AGENT_ENDPOINT>
          jaeger agent endpoint
      --node <NODE>
          node network address looks like localhost:40002:node2:k8s_cluster1 last slice is optional, none means not k8s env
      --is-danger
          is chain in danger mode
      --disable-metrics
          disable metrics
```

说明：
1. `node`为必选参数。值为节点的网络地址,包含`ip`,`port`，`domain`，`cluster name`。
2. `cluster name`是节点所在的`k8s`集群的名称。出于兼容性考虑，非`k8s`环境该项可以省略。


```
$ cloud-config append-k8s --node localhost:40002:node2:k8s
node_address: 1c35eecbba4619ae3edf6c0ea48c4ebdba5a85ed validator_address: 1c35eecbba4619ae3edf6c0ea48c4ebdba5a85ed
$ ls test-chain-node*
test-chain-node0:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-node1:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-node2:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address
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
$ ls test-chain-node*
test-chain-node0:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address

test-chain-node1:
accounts  ca_cert  certs  chain_config.toml  config.toml  node_address  node_config.toml  private_key  validator_address
```
