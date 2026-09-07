#!/bin/bash
set -e

ACTION=${1:-build}
BUILD_DIR="build"

if [ "$ACTION" = "build" ]; then
    if [ "$RAD_MULTILIB" = "1" ]; then
        CONFIG_FLAGS="--enable-multilib --with-multilib-list=m64,m32"
        echo "--- Building GCC with Multilib ($CONFIG_FLAGS) ---"
    else
        CONFIG_FLAGS="--disable-multilib"
        echo "--- Building GCC without Multilib ---"
    fi
    sed -i 's/char [*]q/const &/' libgomp/affinity-fmt.c
    sed -e '/m64=/s/lib64/lib/' \
    -e '/m32=/s/m32=.*/m32=..\/lib32$(call if_multiarch,:i386-linux-gnu)/' \
    -i.orig gcc/config/i386/t-linux64
    sed '/STACK_REALIGN_DEFAULT/s/0/(!TARGET_64BIT \&\& TARGET_SSE)/' \
    -i gcc/config/i386/i386.h
    rm -rf "$BUILD_DIR" && mkdir -p "$BUILD_DIR" && cd "$BUILD_DIR"
    ../configure \
	LD=ld \
        --prefix=/usr \
        --libexecdir=/usr/lib \
        --enable-languages=c,c++ \
        --with-system-zlib \
	--enable-default-pie \
	--enable-default-ssp \
	--enable-host-pie \
        --enable-shared \
        --enable-threads=posix \
        --disable-bootstrap \
	--disable-fixincludes \
        --disable-libstdcxx-pch \
        $CONFIG_FLAGS
    make -j$RAD_CORES
elif [ "$ACTION" = "install" ]; then
    cd $BUILD_DIR
    make DESTDIR="$DESTDIR" install
    mkdir $DESTDIR/usr/lib -p
    ln -svr $DESTDIR/usr/bin/cpp $DESTDIR/usr/lib/cpp
    mkdir $DESTDIR/usr/lib/bfd-plugins -p
    ln -sfv ../../libexec/gcc/$(gcc -dumpmachine)/15.2.0/liblto_plugin.so \
    $DESTDIR/usr/lib/bfd-plugins/
    mkdir $DESTDIR/usr/share/gdb/auto-load/usr/lib -p
    mv -v $DESTDIR/usr/lib/*gdb.py $DESTDIR/usr/share/gdb/auto-load/usr/lib
fi
