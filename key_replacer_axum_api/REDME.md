# 数据库检索替换服务

> 通过检索指定数据表的关键词，将文章内容相关词替换为带链接的内容。

## 相关资料

1. postman.json: 接口文档

## 关于macOS + Jebrains Intelij Idea准备运行环境

```shell
# 首先安装mysql-client
brew install mysql-client
# 然后配置环境变量
export LIBRARY_PATH=/usr/local/opt/mysql-client/lib:$LIBRARY_PATH
# 接着cargo才能从环境变量找到mysql对应的库
cargo install diesel_cli
echo DATABASE_URL=postgres://username:password@localhost/diesel_demo > .env
diesel setup
diesel migration generate create_posts
# up.sql/down.sql添加sql
diesel migration run # 执行迁移
diesel migration redo # 撤销迁移
# 打印schema
diesel print-schema > src/schema.rs
```

> 在idea里面运行时，还需要添加环境变量

```shell
LIBRARY_PATH=/usr/local/opt/mysql-client/lib:$LIBRARY_PATH
```