FROM fedora:33
RUN dnf -y install gcc ccache make meson \
	desktop-file-utils gtk3-devel \
	rust rustfmt cargo \
	alsa-lib-devel openssl-devel pulseaudio-libs-devel
RUN meson target
RUN cargo fmt -- --check && \
	meson install -C target && \
	meson test -C target