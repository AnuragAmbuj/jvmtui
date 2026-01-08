Name:           jvm-tui
Version:        0.1.0
Release:        1%{?dist}
Summary:        Beautiful TUI for JVM monitoring

License:        MIT OR Apache-2.0
URL:            https://github.com/AnuragAmbuj/jvmtui
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.75
BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  openssl-devel
BuildRequires:  pkg-config

Requires:       openssl-libs

# Recommend JDK with command-line tools
Recommends:     java-21-openjdk-devel
Recommends:     java-17-openjdk-devel
Recommends:     java-11-openjdk-devel

%description
Jvmtui brings JVM monitoring to your terminal with a keyboard-driven
interface. Monitor heap usage, garbage collection, memory pools, and threads in
real-time - perfect for SSH sessions and production environments where GUI tools
aren't available.

Features:
- Auto-discovery of running JVMs
- Real-time heap usage, GC statistics, and memory pool metrics
- Keyboard-driven interface with Vim-style navigation
- No JVM agents required - uses standard JDK tools (jcmd, jstat, jps)
- SSH-friendly and lightweight

This package requires JDK 11+ with command-line tools (jcmd, jstat, jps)
to be installed on the system.

%prep
%autosetup

%build
export CARGO_HOME=$(pwd)/.cargo
cargo build --release --locked

%install
rm -rf $RPM_BUILD_ROOT
install -D -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}

%files
%license LICENSE-MIT LICENSE-APACHE
%doc README.md
%{_bindir}/%{name}

%changelog
* Thu Jan 08 2026 Anurag Ambuj <anuragambuj@users.noreply.github.com> - 0.1.0-1
- Initial RPM release
- Complete Phase 1 MVP with real-time JVM monitoring
- Features: heap usage, GC statistics, memory pools, threads
- Auto-discovery of running JVMs
- Keyboard-driven TUI interface
- No JVM agents required
