NAME := rtree
MODE ?= debug
VERSION ?= 0.9.5

TARGET ?= ${NAME}
TARGET_PATH := target/${MODE}/${TARGET}
DESTINATION_PATH := ${TARGET}-${VERSION}-${MODE}

PREFIX ?= /usr/local
INSTALL ?= install

MAN_DIR ?= ${PREFIX}/share/man/man1
MANPAGE_NAME := ${NAME}.1

OS ?= $(shell uname -s)

# 判断系统类型
ifeq ($(OS),Linux)
	OS_TYPE := Linux
else ifeq ($(OS),Darwin)
	OS_TYPE := Macos
else ifeq ($(OS),Windows_NT)
	OS_TYPE := Windows
else
	OS_TYPE := Unknown
endif

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
ifeq (${OS_TYPE}, Linux)
	@echo Linux detected, copy ${TARGET_PATH} to ./${DESTINATION_PATH}
	@cp ${TARGET_PATH} ${DESTINATION_PATH}
else ifeq ($(OS_TYPE), Windows)
	@echo Windows detected, copy ${TARGET_PATH}.exe to ./${DESTINATION_PATH}.exe
	@powershell -Command "Copy-Item ${TARGET_PATH}.exe -Destination ${DESTINATION_PATH}.exe"
else ifeq ($(OS_TYPE), Macos)
	@echo Macos detected, copy ${TARGET_PATH} to ./${DESTINATION_PATH}
	@cp ${TARGET_PATH} ${DESTINATION_PATH}
else
	@echo "Unknown OS is not supported yet."
endif

# 测试目标
test: build
	@echo Start to Test ${NAME}, Version ${VERSION}
	@echo ------ Test1 HELP-TEST ------
	@./${NAME} -h
	@echo ------ Test2 VERSION-TEST ------
	@./${NAME} -V
	@echo ------ Test3 DIRECTORY-ONLY-TEST ------
	@./${NAME} -d
	@echo ------ Test4 FILE-ONLY-TEST ------
	@./${NAME} --fileonly
	@echo ------ Test5 ALL-LIST-TEST ------
	@./${NAME} -a
# TODO:

# 安装目标
install: build
ifeq (${OS_TYPE}, Linux)
# 安装可执行文件
	@sudo $(INSTALL) -d $(PREFIX)/bin
	sudo $(INSTALL) -m 755 ${TARGET_PATH} $(PREFIX)/bin
# 安装 man 页面
	@sudo $(INSTALL) -d $(MAN_DIR)
	@sudo $(INSTALL) -m 644 $(MANPAGE_NAME) $(MAN_DIR)
else ifeq (${OS_TYPE}, Windows)
	@echo "INSTALL: Windows OS is not supported yet."
else
	@echo "ISNTALL: Unknown OS is not supported yet."
endif

clean:
	@cargo clean

.PHONY: build test install clean
