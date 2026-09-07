#!/bin/bash
set -e

ACTION=${1:-build}
BUILD_DIR=$([ "$IS_M32" = "1" ] && echo "build32" || echo "build")

if [ "$ACTION" = "build" ]; then
    if [ ! -f .patched ]; then
        wget -q https://www.linuxfromscratch.org/patches/lfs/12.4/glibc-2.42-fhs-1.patch
        patch -Np1 -i ./glibc-2.42-fhs-1.patch || true
        sed -e '/unistd.h/i #include <string.h>' \
            -e '/libc_rwlock_init/c __libc_rwlock_define_initialized (, reset_lock); memcpy (\&lock, \&reset_lock, sizeof (lock));' \
            -i stdlib/abort.c
        touch .patched
    fi
    rm -rf "$BUILD_DIR"
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"
    echo 'rootsbindir=/usr/sbin' > configparms

    if [ "$IS_M32" = "1" ]; then
        export CC="gcc -m32"
        export CXX="g++ -m32"
        GCC_VER=$(gcc -dumpversion)
        export LIBRARY_PATH="/usr/lib/gcc/x86_64-pc-linux-gnu/${GCC_VER}/32:${LIBRARY_PATH:-}"
        T_HOST="i686-pc-linux-gnu"
        T_LIBDIR="$LIBDIR"
    else
        T_HOST="x86_64-pc-linux-gnu"
        T_LIBDIR="$LIBDIR"
    fi

    ../configure \
        --prefix="$PREFIX" \
        --host="$T_HOST" \
        --build="$(../scripts/config.guess)" \
        --libdir="$T_LIBDIR" \
        --libexecdir="$T_LIBDIR" \
        --disable-werror \
        --disable-nscd \
        --enable-stack-protector=strong \
        --enable-kernel=5.4 \
        libc_cv_slibdir="$T_LIBDIR"

    make -j$RAD_CORES

elif [ "$ACTION" = "install" ]; then
    make -C "$BUILD_DIR" DESTDIR="$DESTDIR" install
fi
