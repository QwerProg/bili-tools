class Bt < Formula
  desc "B站直播开播工具 — 命令行一键开播/下播"
  homepage "https://github.com/QwerProg/bili-tools"
  license "MIT"
  head "https://github.com/QwerProg/bili-tools.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end
end
