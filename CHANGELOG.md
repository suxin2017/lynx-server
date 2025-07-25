# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2025-07-26

### 🚀 Features

- Better content modify
- Api debug tree ([#46](https://github.com/suxin2017/lynx-server/issues/46))

## [0.3.8] - 2025-07-26

### 🚀 Features

- Pwa support
- Filter template
- *(apiDebug)* 添加响应覆盖功能按钮并优化布局样式

### 🚜 Refactor

- *(apiDebug)* 优化请求历史记录的分页和状态管理

## [0.3.7] - 2025-07-25

### 🚀 Features

- General setting ([#44](https://github.com/suxin2017/lynx-server/issues/44))

### 💼 Other

- Ui layout ([#43](https://github.com/suxin2017/lynx-server/issues/43))

### 🚜 Refactor

- Setting config ([#45](https://github.com/suxin2017/lynx-server/issues/45))

## [0.3.6] - 2025-07-15

### 🚀 Features

- Batch handler rule ([#34](https://github.com/suxin2017/lynx-server/issues/34))
- Add import and export rules
- Supports using SSE to retrieve request logs. ([#41](https://github.com/suxin2017/lynx-server/issues/41))

### 🐛 Bug Fixes

- Compression bug ([#36](https://github.com/suxin2017/lynx-server/issues/36))
- Network request traceid error ([#39](https://github.com/suxin2017/lynx-server/issues/39))
- Proxy forward ui ([#40](https://github.com/suxin2017/lynx-server/issues/40))
- Scroll does not reach the bottom
- Dead lock

## [0.3.3] - 2025-07-01

### 🚀 Features

- Support request delay ([#30](https://github.com/suxin2017/lynx-server/issues/30))

## [0.3.1] - 2025-07-01

### 🚀 Features

- Support client proxy ([#32](https://github.com/suxin2017/lynx-server/issues/32))

## [0.3.0] - 2025-06-27

### 🚀 Features

- *(core)* Support api request ([#27](https://github.com/suxin2017/lynx-server/issues/27))

## [0.2.5] - 2025-06-21

### 🚀 Features

- Better display of status time ([#28](https://github.com/suxin2017/lynx-server/issues/28))
- Support inject script ([#29](https://github.com/suxin2017/lynx-server/issues/29))

## [0.2.4] - 2025-06-13

### 🚀 Features

- Pwa and one click add block rule ([#23](https://github.com/suxin2017/lynx-server/issues/23))
- Support jaeger log ([#25](https://github.com/suxin2017/lynx-server/issues/25))

## [0.2.3] - 2025-06-10

### 🐛 Bug Fixes

- Daemons don't behave as expected with empty bodies   ([#20](https://github.com/suxin2017/lynx-server/issues/20))

### 🚜 Refactor

- Refactor Network UI ([#16](https://github.com/suxin2017/lynx-server/issues/16))

## [0.2.2] - 2025-06-08

### 🚀 Features

- The command line supports daemons ([#15](https://github.com/suxin2017/lynx-server/issues/15))

## [0.2.1] - 2025-06-05

### 🐛 Bug Fixes

- Dark mode bug

## [0.2.0] - 2025-06-05

### 🚀 Features

- Proxy Interception Support ([#8](https://github.com/suxin2017/lynx-server/issues/8))

## [0.1.7] - 2025-05-26

### 🐛 Bug Fixes

- Cli start error

## [0.1.6] - 2025-05-26

### 🐛 Bug Fixes

- Table style and websocket log ([#6](https://github.com/suxin2017/lynx-server/issues/6))
- Record time error ([#7](https://github.com/suxin2017/lynx-server/issues/7))
- Record time error
- Test case

## [0.1.5] - 2025-05-22

### 🚀 Features

- Support a more user-friendly experience for rule config and network tree  ([#39](https://github.com/suxin2017/lynx-server/issues/39))
- Support html,css,js,font,video,image,font content preview ([#41](https://github.com/suxin2017/lynx-server/issues/41))
- Filter support and limit the number of size ([#42](https://github.com/suxin2017/lynx-server/issues/42))
- Websocket support ([#43](https://github.com/suxin2017/lynx-server/issues/43))
- Add some layer ([#46](https://github.com/suxin2017/lynx-server/issues/46))
- Add axum and swagger ([#47](https://github.com/suxin2017/lynx-server/issues/47))
- Add request session event ([#2](https://github.com/suxin2017/lynx-server/issues/2))

### 🐛 Bug Fixes

- A lot of bugs

### 🚜 Refactor

- Refactoring everything ([#44](https://github.com/suxin2017/lynx-server/issues/44))

## [0.1.4] - 2025-02-17

### 🐛 Bug Fixes

- Unable to create dir on startup  ([#36](https://github.com/suxin2017/lynx-server/issues/36))
- Http1.1, http 1.0 proxy request and lose some header ([#34](https://github.com/suxin2017/lynx-server/issues/34))

## [0.1.3] - 2025-02-15

### 🐛 Bug Fixes

- Window local ip ([#33](https://github.com/suxin2017/lynx-server/issues/33))
- *(ui)* Clear request log and content ui bug in request tree struce  ([#35](https://github.com/suxin2017/lynx-server/issues/35))

## [0.1.2] - 2025-02-14

### 🚜 Refactor

- Use include dir replace static dir ([#32](https://github.com/suxin2017/lynx-server/issues/32))

## [0.1.1] - 2025-02-14

### 🐛 Bug Fixes

- Ui assert not found

## [0.1.0] - 2025-02-13

### 🚀 Features

- Rule support
- Add rule group
- Support tariui ([#5](https://github.com/suxin2017/lynx-server/issues/5))
- *(lynx-core)* Support glob match model 
- Support more access ip
- Support certificate download and install doc
- Fetch request log in the app context ([#13](https://github.com/suxin2017/lynx-server/issues/13))
- Support clear request log ([#16](https://github.com/suxin2017/lynx-server/issues/16))
- Support ssl capture switch and ssl capture rule ([#18](https://github.com/suxin2017/lynx-server/issues/18))
- Support better default config dir and support specifying dir ([#21](https://github.com/suxin2017/lynx-server/issues/21))

### 🐛 Bug Fixes

- Parse request log to json

<!-- generated by git-cliff -->
