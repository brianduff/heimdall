class HeimdallBin < Formula
  version '0.2'
  desc "Guard the Bifrost."
  homepage "https://github.com/brianduff/heimdall"

  if OS.mac?
      url "https://github.com/brianduff/heimdall/releases/download/v#{version}/heimdall-#{version}.tar.gz"
      sha256 "66facc04e11bb0fab52efc140bee376e8299f01011ae9ff2502bc482673a2201"
  end

  def install
    bin.install "heimdall"
    etc.install "etc/heimdall"
  end
end