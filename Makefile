NAME := rtree
MODE ?= debug
VERSION ?= 0.0.1
TARGET ?= ${NAME}
TARGET_PATH := target/${MODE}/${TARGET}

# 是否是release版本
ifeq (${MODE}, release)
	BUILD_CMD := cargo build --release
else
	BUILD_CMD = cargo build
endif

# 构建目标
build:
	@${BUILD_CMD}
	@cp ${TARGET_PATH} ./

# 测试目标
test: build
	@echo Start to Test ${NAME}, Version ${VERSION}
	@echo ------ Test1 HELP-TEST ------
	@./${NAME} -h

# 安装目标
install: build
	@sudo cp ${TARGET_PATH} /usr/bin/
