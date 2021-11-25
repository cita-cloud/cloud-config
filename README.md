# cloud-config

创建链的配置文件。

### 依赖

* rust: 1.54.0

### 安装

```
cargo install --path .
```

### 用法

```
$ ./target/debug/cloud-config -h                                                                           
cloud-config 6.3.0

Rivtower Technologies.

USAGE:
    cloud-config <SUBCOMMAND>

FLAGS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    append               append config
    append-node          append a node into chain
    append-validator     append a validator into chain
    create               create config
    create-ca            create CA
    create-csr           create csr
    delete               delete config
    delete-chain         delete a chain
    delete-node          delete a node from chain
    help                 Print this message or the help of the given subcommand(s)
    init-chain           init a chain
    init-chain-config    init chain config
    init-node            init node
    new-account          new account
    set-admin            set admin of chain
    set-nodelist         set node list
    set-validators       set validators of chain
    sign-csr             sign csr
    update-node          update node
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

将创建链配置的过程拆分成8个子命令和4个辅助的子命令，以实现最正规的去中心化配置流程。

3个辅助子命令为：

1. [create-ca](/src/create_ca.rs)创建链的根证书。会在`$(config-dir)/$(chain-name)/ca_cert/`下生成`cert.pem`和`key.pem`两个文件。
2. [create-csr](/src/create_csr.rs)为各个节点创建证书和签名请求。会在`$(config-dir)/$(chain-name)/certs/$(domain)/`下生成`csr.pem`和`key.pem`两个文件。
3. [sign-csr](/src/sign_csr.rs)处理节点的签名请求。会在`$(config-dir)/$(chain-name)/certs/$(domain)/`下生成`cert.pem`。
4. [new-account](/src/new_account.rs)创建账户。会在`$(config-dir)/$(chain-name)/accounts/`下，创建以账户地址为名的文件夹，里面有`key_id`和`kms.db`两个文件。

8个子命令分别为：
1. [init-chain](/src/init_chain.rs)。根据指定的`config-dir`和`chan-name`,初始化一个链的文件目录结构。
    ```
    $(config_dir)
    --  $(chain_name)
    ------  accounts
    ------  ca_cert
    ------  certs
    ```
2. [init-chain-config](/src/init_chain_config.rs)。初始化除`admin`(管理员账户)，`validators`(共识节点地址列表)，`node_network_address_list`（节点网络地址列表）之外的`链级配置`。因为前述三个操作需要一些额外的准备工作，且需要先对除此之外的链接配置信息在所有参与方之间达成共识。因此对于去中心化场景来说，这一步其实是一个公示的过程。执行之后会生成`$(config-dir)/$(chain-name)/chain_config.toml`
3. [set-admin](/src/set_admin.rs)。设置管理员账户。账户需要事先通过[new-account](/src/new_account.rs)子命令创建。如果网络微服务选择了`network_tls`，则还需要通过[create-ca](/src/create_ca.rs)创建链的根证书。
4. [set-validators](/src/set_validators.rs)。设置共识节点账户列表。账户同样需要事先通过[new-account](/src/new_account.rs)子命令，由各个共识节点分别创建，然后将账户地址集中到一起进行设置。
5. [set-nodelist](/src/set_nodelist.rs)。设置节点网络地址列表。各个节点参与方需要根据自己的网络环境，预先保留节点的`ip`，`port`和`domain`。然后将相关信息集中到一起进行设置。至此，`链级配置`信息设置完成，可以下发配置文件`chain_config.toml`到各个节点。如果网络微服务选择了`network_tls`，则需要通过`create-csr`根据节点的`domain`为各个节点创建证书和签名请求。然后请求`CA`通过`sign-crs`处理签名请求，并下发生成的`cert.pem`到各个节点。
6. [init-node](/src/init_node.rs)。设置`节点配置`信息。这步操作由各个节点的参与方独立设置，节点之间可以不同。执行之后会生成`$(config-dir)/$(chain-name)-$(domain)/node_config.toml`
7. [update-node](/src/update_node.rs)。根据之前设置的`链级配置`和`节点配置`，生成每个节点所需的微服务配置文件。
8. [delete-chain](/src/delete_chain.rs)删除链。删除属于该链的所有文件夹以及其中的文件，`使用时要慎重`。

在前述流程的基础上，可以封装更高级更方便使用的子命令。比如针对开发测试，用户可以只传递所需的节点数量，其他信息都使用约定好的默认值，无需执行这么多子命令，也无需传递大量参数就可以直接生成一条链。


### 使用示例

#### init-chain

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
```

```
$ ./target/debug/cloud-config init-chain   
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

        --kms_image <KMS_IMAGE>
            set kms micro service image name (kms_eth/kms_sm) [default: kms_sm]

        --kms_tag <KMS_TAG>
            set kms micro service image tag [default: latest]

        --network_image <NETWORK_IMAGE>
            set network micro service image name (network_tls/network_p2p) [default: network_p2p]

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
$ ./target/debug/cloud-config init-chain-config

$ tree test-chain
test-chain
├── accounts
├── ca_cert
├── certs
└── chain_config.toml

$ cat test-chain/chain_config.toml 
node_network_address_list = []

[[micro_service_list]]
image = 'network_p2p'
tag = 'latest'

[[micro_service_list]]
image = 'consensus_raft'
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
timestamp = 1637647692548

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
$ ./target/debug/cloud-config new-account   
key_id:1, address:aeaa6e333b8ed911f89acd01e88e3d9892da87b5

$ ./target/debug/cloud-config new-account
key_id:1, address:1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa

$ ./target/debug/cloud-config new-account
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
./target/debug/cloud-config set-admin --admin aeaa6e333b8ed911f89acd01e88e3d9892da87b5

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
$ ./target/debug/cloud-config set-validators --validators 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa,344a9d7c390ea5f884e7c0ebf30abb17bd8785cd

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

#### set-nodelist

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --nodelist <NODE_LIST>       node list looks like
                                     localhost:40000:node0,localhost:40001:node1
```

说明：

1. `nodelist`为必选参数。值为多个节点的网络地址,用逗号分隔。每个节点的网络地址包含`ip`,`port`和`domain`，之间用分号分隔。
2. `domain`为任意字符串，只需要确保节点之间不重复即可。

```
$ ./target/debug/cloud-config set-nodelist --nodelist localhost:40000:node0,localhost:40001:node1 

$ cat test-chain/chain_config.toml | grep -A3 node_network                                       
[[node_network_address_list]]
domain = 'node0'
host = 'localhost'
port = 40000
--
[[node_network_address_list]]
domain = 'node1'
host = 'localhost'
port = 40001
```

#### append-node

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --node <NODE>                node network address looks like localhost:40002:node2
```

1. `node`为必选参数。值为节点的网络地址包含`ip`,`port`和`domain`，之间用分号分隔。
2. 功能与`set-nodelist`相似，只不过是每次添加一个节点。


#### delete-node

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
        --domain <DOMAIN>            domain of node that want to delete
```

1. `domain`为必选参数。作为节点的标识，标识要删除的节点。
2. 功能与`append-node`相反，删除添加一个节点。

#### create-ca

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
```

```
$ ./target/debug/cloud-config create-ca

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
$ ./target/debug/cloud-config create-csr --domain node0 

$ ./target/debug/cloud-config create-csr --domain node1

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
$ ./target/debug/cloud-config sign-csr --domain node0 

$ ./target/debug/cloud-config sign-csr --domain node1

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
            key id of account in kms db [default: info]

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

```
$ ./target/debug/cloud-config init-node --domain node0 
 
$ ./target/debug/cloud-config init-node --domain node1

$ tree test-chain-node* 
test-chain-node0
└── node_config.toml
test-chain-node1
└── node_config.toml

$ cat test-chain-node0/node_config.toml 
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
        --account <ACCOUNT>            account of node
        --chain-name <CHAIN_NAME>      set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>      set config file directory, default means current directory
                                       [default: .]
        --config-name <CONFIG_NAME>    set config file name [default: config.toml]
        --domain <DOMAIN>              domain of node
```

说明：
1. `domain`为必选参数，作为节点的标识，表示要操作的节点。
2. `account`为必选参数，表示该节点要使用的账户地址。值为之前用`new-account`创建的地址。

```
$ ./target/debug/cloud-config update-node --domain node0 --account 1b3b5e847f5f4a7ff2842f1b0c72a8940e4adcfa

$ ./target/debug/cloud-config update-node --domain node1 --account 344a9d7c390ea5f884e7c0ebf30abb17bd8785cd

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

#### delete-chain

参数：

```
        --chain-name <CHAIN_NAME>    set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>    set config file directory, default means current directory
                                     [default: .]
```

说明：
1. 该命令会删除所有跟指定链相关的文件夹及文件，`使用时要慎重`。

### 初始化链

```
$ config-config create -h
cloud-config-create 

create config

USAGE:
    cloud-config create [OPTIONS] --consensus <CONSENSUS>

FLAGS:
    -h, --help    Print help information
        --use_num    use serial number instead of node address

OPTIONS:
        --chain-name <CHAIN_NAME>
            set chain name [default: test-chain]

        --config-dir <CONFIG_DIR>
            set config file directory, default means current directory

        --config-name <CONFIG_NAME>
            set config file name [default: config.toml]

        --consensus <CONSENSUS>
            Set consensus micro-service

        --controller <CONTROLLER>
            Set controller micro-service [default: controller]

        --executor <EXECUTOR>
            Set executor micro-service [default: executor_evm]

        --grpc-ports <GRPC_PORTS>
            grpc port list, input "p1,p2,p3,p4", use default grpc port count from 50000 + 1000 * i
            use default must set peer_count or p2p_ports [default: default]

        --kms <KMS>
            Set kms micro-service [default: kms_sm]

        --kms-password <KMS_PASSWORD>
            kms db password [default: 123456]

        --network <NETWORK>
            Set network micro-service

        --node-list <NODE_LIST>
            input "localhost:40000:node0,localhost:40001:node1", use default port count from
            127.0.0.1:40000 + 1 * i:nodei, use default must set peer_count or grpc_ports

        --package-limit <PACKAGE_LIMIT>
            set one block contains tx limit, default 30000 [default: 30000]

        --peers-count <PEERS_COUNT>
            set initial node number, default "none" mean not use this must set grpc_ports or
            p2p_ports, if set peers_count, grpc_ports and p2p_ports, base on grpc_ports > p2p_ports
            > peers_count

        --storage <STORAGE>
            Set storage micro-service [default: storage_rocksdb]

        --version <VERSION>
            set version [default: 0]
```

#### 例子

```
$ cloud-config create --peers-count 4 --network network_p2p --consensus consensus_raft --config-dir /tmp/test
```

#### 生成的文件

```
$ cd /tmp/test 
$ ls
test-chain                                           test-chain-57b98686b6636877a04710dc57127526feac76e7  test-chain-b4d2011d32ff5484b18dcb237e0dbf504b11c97e
test-chain-3e3acf2feb25ac611db9348244de132d01327dab  test-chain-94cc5493111435bcfb0a03eb39921ad0f2e379f8

$ tree .
.
├── test-chain
│   ├── config.toml
│   └── kms.db
├── test-chain-3e3acf2feb25ac611db9348244de132d01327dab
│   ├── config.toml
│   ├── consensus-log4rs.yaml
│   ├── controller-log4rs.yaml
│   ├── executor-log4rs.yaml
│   ├── kms.db
│   ├── kms-log4rs.yaml
│   ├── network-log4rs.yaml
│   └── storage-log4rs.yaml
├── test-chain-57b98686b6636877a04710dc57127526feac76e7
│   ├── config.toml
│   ├── consensus-log4rs.yaml
│   ├── controller-log4rs.yaml
│   ├── executor-log4rs.yaml
│   ├── kms.db
│   ├── kms-log4rs.yaml
│   ├── network-log4rs.yaml
│   └── storage-log4rs.yaml
├── test-chain-94cc5493111435bcfb0a03eb39921ad0f2e379f8
│   ├── config.toml
│   ├── consensus-log4rs.yaml
│   ├── controller-log4rs.yaml
│   ├── executor-log4rs.yaml
│   ├── kms.db
│   ├── kms-log4rs.yaml
│   ├── network-log4rs.yaml
│   └── storage-log4rs.yaml
└── test-chain-b4d2011d32ff5484b18dcb237e0dbf504b11c97e
    ├── config.toml
    ├── consensus-log4rs.yaml
    ├── controller-log4rs.yaml
    ├── executor-log4rs.yaml
    ├── kms.db
    ├── kms-log4rs.yaml
    ├── network-log4rs.yaml
    └── storage-log4rs.yaml

5 directories, 34 files
```

`test-chain-b4d2011d32ff5484b18dcb237e0dbf504b11c97e`：节点名称的构造为 `<chain-name>-<node-address>`

### 增加节点

```
$ cloud-config append -h
cloud-config-append 

append config

USAGE:
    cloud-config append [OPTIONS]

FLAGS:
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
        --chain-name <CHAIN_NAME>
            set chain name [default: test-chain]

        --config-dir <CONFIG_DIR>
            set config file directory, default means current directory

        --config-name <CONFIG_NAME>
            set config file name [default: config.toml]

        --grpc-ports <GRPC_PORTS>
            grpc port list, input "p1,p2,p3,p4", use default grpc port count from 50000 + 1000 * i
            use default must set peer_count or p2p_ports [default: default]

        --kms-password <KMS_PASSWORD>
            kms db password [default: 123456]

        --network <NETWORK>
            Set network micro-service

        --p2p-ports <P2P_PORTS>
            p2p port list, input "ip1:port1,ip2:port2,ip3:port3,ip4:port4", use default port count
            from 127.0.0.1:40000 + 1 * i, use default must set peer_count or grpc_ports [default:
            default]

        --package-limit <PACKAGE_LIMIT>
            set one block contains tx limit, default 30000 [default: 30000]

        --peers-count <PEERS_COUNT>
            set initial node number, default "none" mean not use this must set grpc_ports or
            p2p_ports, if set peers_count, grpc_ports and p2p_ports, base on grpc_ports > p2p_ports
            > peers_count
```

##### 例子

```
$ cloud-config append --peers-count 2 --config-dir /tmp/test

$ ls /tmp/test
test-chain-3e3acf2feb25ac611db9348244de132d01327dab  test-chain-57b98686b6636877a04710dc57127526feac76e7  test-chain-b4d2011d32ff5484b18dcb237e0dbf504b11c97e
test-chain  test-chain-4f9cec049857a119b51472dac520f4bde0bca4d0  test-chain-94cc5493111435bcfb0a03eb39921ad0f2e379f8  test-chain-e51ab390ec569bb5e9b0c96a84df40b9f6923af2
```

增加了2个新节点 `test-chain-4f9cec049857a119b51472dac520f4bde0bca4d0`，`test-chain-e51ab390ec569bb5e9b0c96a84df40b9f6923af2`

### 减少节点

```
$ cloud-config delete -h
cloud-config-delete 

delete config

USAGE:
    cloud-config delete [OPTIONS]

FLAGS:
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
        --addresses <ADDRESSES>        delete node address. Such as
                                       0x16e80b488f6e423b9faff014d1883493c5043d29,0x5bc21f512f877f18840abe13de5816c1226c4710
                                       will node with the address [default: default]
        --chain-name <CHAIN_NAME>      set chain name [default: test-chain]
        --config-dir <CONFIG_DIR>      set config file directory, default means current directory
        --config-name <CONFIG_NAME>    set config file name [default: config.toml]
        --index <INDEX>                delete index. Such as 1,2 will delete first and second node
                                       [default: default]
```

#### 例子

```
$ cloud-config delete --addresses 0x4f9cec049857a119b51472dac520f4bde0bca4d0,0xe51ab390ec569bb5e9b0c96a84df40b9f6923af2 --config-dir /tmp/test

$ ls /tmp/test
test-chain                                           test-chain-57b98686b6636877a04710dc57127526feac76e7  test-chain-b4d2011d32ff5484b18dcb237e0dbf504b11c97e
test-chain-3e3acf2feb25ac611db9348244de132d01327dab  test-chain-94cc5493111435bcfb0a03eb39921ad0f2e379f8
```

节点`test-chain-4f9cec049857a119b51472dac520f4bde0bca4d0`，`test-chain-e51ab390ec569bb5e9b0c96a84df40b9f6923af2`被删除

### 
