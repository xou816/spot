FROM fedora:33
RUN dnf -y install gcc ccache make meson \
	desktop-file-utils gtk3-devel \
	rust rustfmt cargo \
	alsa-lib-devel openssl-devel pulseaudio-libs-devel
COPY entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]