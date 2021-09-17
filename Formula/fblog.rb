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

  depends_on "pkg-config" => :build
  depends_on "rust" => :build

  def install
    system "cargo", "install", "--all-features", *std_cargo_args

    # Completion scripts and manpage are generated in the crate's build
    # directory, which includes a fingerprint hash. Try to locate it first
    out_dir = Dir["target/release/build/fblog-*/out"].first
  end

end
