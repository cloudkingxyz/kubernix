let
  moz_overlay = import (
    builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz
  );
  pkgs = import ./nixpkgs.nix {
    overlays = [ moz_overlay ];
  };
  ruststable = (pkgs.latest.rustChannels.stable.rust.override {
    extensions = [
      "clippy-preview"
      "rustfmt-preview"
    ];
  });
in
pkgs.stdenv.mkDerivation {
  buildInputs = with pkgs; [
    bash
    cacert
    cfssl
    conmon
    conntrack-tools
    cni-plugins
    cri-o
    cri-tools
    etcd
    git
    iproute
    iptables
    kubernetes
    runc
    ruststable
    socat
    utillinux
  ];

  LANG = "en_US.UTF-8";

  shellHook = ''
    export CONTAINER_RUNTIME_ENDPOINT="unix://$PWD/run/crio/crio.sock"
    export KUBECONFIG="run/kube/admin.kubeconfig"
  '';

  name = "shell";
}
