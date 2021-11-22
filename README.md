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
    create               create config
    create-ca            create CA
    create-cert          create cert
    delete               delete config
    delete-chain         delete a chain
    help                 Print this message or the help of the given subcommand(s)
    init-chain           init a chain
    init-chain-config    init chain config
    init-node            init node
    new-account          new account
    set-admin            set admin of chain
    set-nodelist         set node list
    set-validators       set validators of chain
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

整个过程分为：
1. [init-chain](/src/init_chain.rs)。根据用户指定的`config-dir`和`chan-name`,初始化一个链的文件目录结构。
2. [init-chain-config](/src/init_chain_config.rs)。初始化除`admin`(管理员账户)，`validators`(共识节点地址列表)，`node_network_address_list`（节点网络地址列表）之外的`链级配置`。因为前述三个操作需要一些准备工作，且需要先对除此之外的链接配置信息在所有参与方之间达成共识。因此对于去中心化场景来说，这一步其实是一个公示的过程。
3. [set-admin](/src/set_admin.rs)。设置管理员账户。账户需要事先通过[new-account](/src/new_account.rs)子命令创建。
4. 如果网络微服务选择了`network_tls`，则需要通过[create-ca](/src/create_ca.rs)创建根证书。
5. [set-validators](/src/set_validators.rs)。设置共识节点账户列表。账户同样需要事先通过[new-account](/src/new_account.rs)子命令创建。
6. [set-nodelist](/src/set_nodelist.rs)。设置节点网络地址列表。各个节点参与方需要根据自己的网络环境，预先保留好节点的`ip`，`port`和域名。至此，`链级配置`信息设置完成，可以下发配置文件`chain_config.toml`到各个节点。
7. 如果网络微服务选择了`network_tls`，则需要通过[create-cert](/src/create_cert.rs)为各个节点创建网络证书。
8. [init-node](/src/init_node.rs)。设置`节点配置`信息。这步操作由各个节点的参与方独立设置，节点之间可以不同。
9. [update-node](/src/update_node.rs)。根据之前设置的`链级配置`和`节点配置`，生成每个节点所需的微服务配置文件。

如果是测试环境，按照前述流程顺序执行下来即可。

对于实际生产部署来说，因为有同步参与方顺序会更加复杂：
1. 链的发起方。执行1-2-3-4，然后等待共识节点提供信息之后执行5-6-7。然后向节点参与方下发`chain_config.toml`以及可能的各节点的证书信息。
2. 各个节点参与方执行1。然后执行`new-account`创建节点账户；准备自己节点的网络信息。将这些信息提供给链的发起人之后，等待下发`chain_config.toml`以及可能的各节点的证书信息。然后执行8-9生成节点配置文件。


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
