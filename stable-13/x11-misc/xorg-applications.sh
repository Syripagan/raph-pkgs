#!/bin/sh
cat > app-7.md5 << "EOF"
36936e5bcf04b982ea87b4556d082061  iceauth-1.0.11.tar.xz
83d943bbb0e3ab868cb0a7438e135544  mkfontscale-1.2.4.tar.xz
b9efe1d21615c474b22439d41981beef  sessreg-1.1.4.tar.xz
5a10223c3305f48bfb3b09e0a7a139d1  setxkbmap-1.3.5.tar.xz
6484cd8ee30354aaaf8f490988f5f6ef  smproxy-1.0.8.tar.xz
9cfdec89ad7bd86bcdfda150ae995955  xauth-1.1.5.tar.xz
37063ccf902fe3d55a90f387ed62fe1f  xcmsdb-1.0.7.tar.xz
f97e81b2c063f6ae9b18d4b4be7543f6  xcursorgen-1.0.9.tar.xz
700556957773d378fa16a65a4406be0a  xdpyinfo-1.4.0.tar.xz
830a54ef3ba338013e06a1b5b012b4bd  xdriinfo-1.0.8.tar.xz
6d2309b05267e31923b11ed2b83f33ae  xev-1.2.7.tar.xz
687e42aa5afaec37f14da3072651c635  xgamma-1.0.8.tar.xz
45c7e956941194e5f06a9c7307f5f971  xhost-1.0.10.tar.xz
8e4d14823b7cbefe1581c398c6ab0035  xinput-1.6.4.tar.xz
b8128ff6816897bd385ca437cd2886ee  xkbcomp-1.5.0.tar.xz
543c0535367ca30e0b0dbcfa90fefdf9  xkbevd-1.1.6.tar.xz
c572508053297094995f1fc4bc681624  xkbutils-1.0.7.tar.xz
294db9393a9d8e6613e1e3dd4fe0273f  xkill-1.0.7.tar.xz
a98d83568d19cc606b114b133d6ed8be  xlsatoms-1.1.5.tar.xz
cf2fc9e9d298c149253cfc77faaa475e  xlsclients-1.1.6.tar.xz
ba2dd3db3361e374fefe2b1c797c46eb  xmessage-1.0.7.tar.xz
4e6b8655260cca88252e86f5431041c0  xmodmap-1.0.12.tar.xz
ab6c9d17eb1940afcfb80a72319270ae  xpr-1.2.0.tar.xz
5ef4784b406d11bed0fdf07cc6fba16c  xprop-1.2.8.tar.xz
211abd989eef11708cc0a4978d101014  xrandr-1.5.4.tar.xz
44894ebd60a40c54ccc44c67ad3b7de8  xrdb-1.2.3.tar.xz
af26f2a7f128f27e0df30ef246687726  xrefresh-1.1.1.tar.xz
95fcb2ae70ee7e7ef7fe70b096b3d254  xset-1.2.6.tar.xz
33c80b7744f9a452bf9e39ea7303650e  xsetroot-1.1.4.tar.xz
5076c46fe4f6da29377e462ad25c893d  xvinfo-1.1.6.tar.xz
ee94a7722c8b9e37a28f1ac0fc371454  xwd-1.0.10.tar.xz
e24406c671ab09a7ab0e13a7d1ef2752  xwininfo-1.1.7.tar.xz
53d99fe7077b162b0cb87189f7ed71ce  xwud-1.0.8.tar.xz
EOF

mkdir app &&
cd app &&
grep -v '^#' ../app-7.md5 | awk '{print $2}' | wget -i- -c \
    -B https://www.x.org/pub/individual/app/ &&
md5sum -c ../app-7.md5

as_root()
{
  if   [ $EUID = 0 ];        then $*
  elif [ -x /usr/bin/sudo ]; then sudo $*
  else                            su -c \\"$*\\"
  fi
}

export -f as_root

bash -e

for package in $(grep -v '^#' ../app-7.md5 | awk '{print $2}')
do
  packagedir=${package%.tar.?z*}
  tar -xf $package
  pushd $packagedir
     ./configure --prefix=/usr
     make
     make DESTDIR=$DESTDIR install
  popd
  rm -rf $packagedir
done

exit

rm -f $DESTDIR/usr/bin/xkeystone
