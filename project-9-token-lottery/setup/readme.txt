项目运行环境
当前脚本会自动：
- 下载最新 Queue。
- 提取当前 Oracle 地址。
- 下载所有对应的 Oracle 快照。
- 选择可访问的 gateway。
- 准备本地 signer 状态。
- 启动带完整 Switchboard 数据的 validator。
第一个终端：
    cd project-9-token-lottery/setup
    ./setup-local.sh
    ./start-validator.sh
第二个终端：
    cd project-9-token-lottery
    anchor test --skip-local-validator
需要说明的是：它依然依赖至少一个外部 Switchboard gateway 在线。因此我能确认“按照当前主网和 gateway 状态已经跑通”，但不能承诺外部服务未来永久不变。
如果以后所有旧 Queue gateway 都关闭，脚本会明确报：
No online Switchboard oracle gateway with a usable signer

视频中 1:43:49 体现