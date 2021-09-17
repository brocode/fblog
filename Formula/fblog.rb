class Fblog < Formula
  desc "JSON log viewer"
  homepage "https://github.com/brocode/fblog/"
  url "https://github.com/brocode/fblog/archive/refs/tags/v3.0.2.tar.gz"
  sha256 "8ca2a2c40b96834c21e7bcdfe90dd9cee1103410934a70209d8a617a748160f7"
  license "Unlicense"
  head "https://github.com/brocode/fblog.git", branch: "master"

  livecheck do
    url :stable
    strategy :github_latest
  end

  bottle do
    sha256 cellar: :any,                 arm64_big_sur: "d3e0ae859dc1e66ebecbc66a8ad1ec2abac59bc707d2305dde66212e71406d36"
    sha256 cellar: :any,                 big_sur:       "a8f2bd6586de9f7aa36eaaefd36777309f9b5d57f01bf33bf022d715fd3dbb89"
    sha256 cellar: :any,                 catalina:      "0edcffa1251002e2747020d62a16ae077bd7aa5fb289d351622e0065c9686c40"
    sha256 cellar: :any,                 mojave:        "b57024c0d221249a1f5eaef1069ac90d44e54afdadb146acd117ae23b7de98c6"
    sha256 cellar: :any_skip_relocation, x86_64_linux:  "34e3140b55f0fb5efb8db70e0709afe091632efaa84465e4c1c9ca3c8afa1bf2"
  end

  depends_on "pkg-config" => :build
  depends_on "rust" => :build

  def install
    system "cargo", "install", "--all-features", *std_cargo_args

    # Completion scripts and manpage are generated in the crate's build
    # directory, which includes a fingerprint hash. Try to locate it first
    out_dir = Dir["target/release/build/fblog-*/out"].first
  end

end
