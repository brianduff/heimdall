class HeimdallBin < Formula
  version '0.4'
  desc "Guard the Bifrost."
  homepage "https://github.com/brianduff/heimdall"

  if OS.mac?
      url "https://github.com/brianduff/heimdall/releases/download/v#{version}/heimdall-#{version}.tar.gz"
      sha256 "6523da8f93bf67e645639c8c9c38c4e2e40d4dabdb3e6aa5d365a9990be14784"
  end

  def install
    bin.install "heimdall"
    etc.install "etc/heimdall"
  end
end