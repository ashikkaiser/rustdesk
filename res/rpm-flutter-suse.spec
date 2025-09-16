Name:       cloudydesk
Version:    1.4.2
Release:    0
Summary:    RPM package
License:    GPL-3.0
URL:        https://cloudydesk.com
Vendor:     cloudydesk <info@cloudydesk.com>
Requires:   gtk3 libxcb1 xdotool libXfixes3 alsa-utils libXtst6 libva2 pam gstreamer-plugins-base gstreamer-plugin-pipewire
Recommends: libayatana-appindicator3-1
Provides:   libdesktop_drop_plugin.so()(64bit), libdesktop_multi_window_plugin.so()(64bit), libfile_selector_linux_plugin.so()(64bit), libflutter_custom_cursor_plugin.so()(64bit), libflutter_linux_gtk.so()(64bit), libscreen_retriever_plugin.so()(64bit), libtray_manager_plugin.so()(64bit), liburl_launcher_linux_plugin.so()(64bit), libwindow_manager_plugin.so()(64bit), libwindow_size_plugin.so()(64bit), libtexture_rgba_renderer_plugin.so()(64bit)

# https://docs.fedoraproject.org/en-US/packaging-guidelines/Scriptlets/

%description
The best open-source remote desktop client software, written in Rust.

%prep
# we have no source, so nothing here

%build
# we have no source, so nothing here

# %global __python %{__python3}

%install

mkdir -p "%{buildroot}/usr/share/cloudydesk" && cp -r ${HBB}/flutter/build/linux/x64/release/bundle/* -t "%{buildroot}/usr/share/cloudydesk"
mkdir -p "%{buildroot}/usr/bin"
install -Dm 644 $HBB/res/cloudydesk.service -t "%{buildroot}/usr/share/cloudydesk/files"
install -Dm 644 $HBB/res/cloudydesk.desktop -t "%{buildroot}/usr/share/cloudydesk/files"
install -Dm 644 $HBB/res/cloudydesk-link.desktop -t "%{buildroot}/usr/share/cloudydesk/files"
install -Dm 644 $HBB/res/128x128@2x.png "%{buildroot}/usr/share/icons/hicolor/256x256/apps/cloudydesk.png"
install -Dm 644 $HBB/res/scalable.svg "%{buildroot}/usr/share/icons/hicolor/scalable/apps/cloudydesk.svg"

%files
/usr/share/cloudydesk/*
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
ln -sf /usr/share/cloudydesk/cloudydesk /usr/bin/cloudydesk
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
    rm /usr/bin/cloudydesk || true
    rmdir /usr/lib/cloudydesk || true
    rmdir /usr/local/cloudydesk || true
    rmdir /usr/share/cloudydesk || true
    rm /usr/share/applications/cloudydesk.desktop || true
    rm /usr/share/applications/cloudydesk-link.desktop || true
    update-desktop-database
  ;;
  1)
    # for upgrade
    rmdir /usr/lib/cloudydesk || true
    rmdir /usr/local/cloudydesk || true
  ;;
esac
