class HeimdallBin < Formula
  version '0.6'
  desc "Guard the Bifrost."
  homepage "https://github.com/brianduff/heimdall"

  if OS.mac?
      url "https://github.com/brianduff/heimdall/releases/download/v#{version}/heimdall-#{version}.tar.gz"
      sha256 "474585e6f07f58b1008634d2318c328642a108d2b8e6753710d0e02c6987f614"
  end

  def install
    bin.install "heimdall"
    etc.install "etc/heimdall"
  end
end