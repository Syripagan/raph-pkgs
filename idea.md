The idea of the
Rathrix's Automated Package Holder
for Rathrix LFS Distribution
------------------------------------

Packages:
raph -i <package>: Installing some package;
raph -d <package>: Clean system from package but keep dependencies;
raph -u <package>: Clone new source and update package;

Repos:
raph -r <link>: Set hosted repository link;

Other:
raph -v: Print version of the RAPH binary;
raph -h: Actually print this menu, but it just need to explain how to use this binary;
raph -c: Set config directory for current command (e.g. "/etc/raph/");

Config:

"
# Raph config example
# ----------------

# This is repo link, where the prebuilt packages are. They will be named as
# <package>-<version>.tar.gz, and raph will install 
# the latest of avaible packages by default, but if version is defined
# it will install this version, pretty simple
[repo]
url = https://github.com/Syripagan/raph

# Then goes prefix, that defines if packages are installing in /usr or other DESTDIR (by default it is /usr)
[prefix]
path = /usr
"
End of config example. Some other options will be added if needed

For now that's the whole idea

