Name:       cloudydesk
Version:    1.1.9
Release:    0
Summary:    RPM package
License:    GPL-3.0
Requires:   gtk3 libxcb1 xdotool libXfixes3 alsa-utils libXtst6 libva2 pam gstreamer-plugins-base gstreamer-plugin-pipewire
Recommends: libayatana-appindicator3-1

# https://docs.fedoraproject.org/en-US/packaging-guidelines/Scriptlets/

%description
The best open-source remote desktop client software, written in Rust.

%prep
# we have no source, so nothing here

%build
# we have no source, so nothing here

%global __python %{__python3}

%install
mkdir -p %{buildroot}/usr/bin/
mkdir -p %{buildroot}/usr/share/cloudydesk/
mkdir -p %{buildroot}/usr/share/cloudydesk/files/
mkdir -p %{buildroot}/usr/share/icons/hicolor/256x256/apps/
mkdir -p %{buildroot}/usr/share/icons/hicolor/scalable/apps/
install -m 755 $HBB/target/release/cloudydesk %{buildroot}/usr/bin/cloudydesk
install $HBB/libsciter-gtk.so %{buildroot}/usr/share/cloudydesk/libsciter-gtk.so
install $HBB/res/cloudydesk.service %{buildroot}/usr/share/cloudydesk/files/
install $HBB/res/128x128@2x.png %{buildroot}/usr/share/icons/hicolor/256x256/apps/cloudydesk.png
install $HBB/res/scalable.svg %{buildroot}/usr/share/icons/hicolor/scalable/apps/cloudydesk.svg
install $HBB/res/cloudydesk.desktop %{buildroot}/usr/share/cloudydesk/files/
install $HBB/res/cloudydesk-link.desktop %{buildroot}/usr/share/cloudydesk/files/

%files
/usr/bin/cloudydesk
/usr/share/cloudydesk/libsciter-gtk.so
/usr/share/cloudydesk/files/cloudydesk.service
/usr/share/icons/hicolor/256x256/apps/cloudydesk.png
/usr/share/icons/hicolor/scalable/apps/cloudydesk.svg
/usr/share/cloudydesk/files/cloudydesk.desktop
/usr/share/cloudydesk/files/cloudydesk-link.desktop

%changelog
# let's skip this for now

%pre
# can do something for centos7
case "$1" in
  1)
    # for install
  ;;
  2)
    # for upgrade
    systemctl stop cloudydesk || true
  ;;
esac

%post
cp /usr/share/cloudydesk/files/cloudydesk.service /etc/systemd/system/cloudydesk.service
cp /usr/share/cloudydesk/files/cloudydesk.desktop /usr/share/applications/
cp /usr/share/cloudydesk/files/cloudydesk-link.desktop /usr/share/applications/
systemctl daemon-reload
systemctl enable cloudydesk
systemctl start cloudydesk
update-desktop-database

%preun
case "$1" in
  0)
    # for uninstall
    systemctl stop cloudydesk || true
    systemctl disable cloudydesk || true
    rm /etc/systemd/system/cloudydesk.service || true
  ;;
  1)
    # for upgrade
  ;;
esac

%postun
case "$1" in
  0)
    # for uninstall
    rm /usr/share/applications/cloudydesk.desktop || true
    rm /usr/share/applications/cloudydesk-link.desktop || true
    update-desktop-database
  ;;
  1)
    # for upgrade
  ;;
esac
