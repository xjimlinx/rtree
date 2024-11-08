NAME := rtree
MODE ?= debug
VERSION ?= 0.0.1

TARGET ?= ${NAME}
TARGET_PATH := target/${MODE}/${TARGET}

PREFIX ?= /usr/local
INSTALL ?= install

MAN_DIR ?= ${PREFIX}/share/man/man1
MANPAGE_NAME := ${NAME}.1

# 是否是release版本
ifeq (${MODE}, release)
	BUILD_CMD := cargo build --release
else
	BUILD_CMD = cargo build
endif

# 编译
all: build

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
# 安装可执行文件
	@sudo $(INSTALL) -d $(PREFIX)/bin
	@sudo $(INSTALL) -m 755 rtree $(PREFIX)/bin
# 安装 man 页面
	@sudo $(INSTALL) -d $(MAN_DIR)
	@sudo $(INSTALL) -m 644 $(MANPAGE_NAME) $(MAN_DIR)

.PHONY: build test install clean