class HeimdallBin < Formula
  version '0.3'
  desc "Guard the Bifrost."
  homepage "https://github.com/brianduff/heimdall"

  if OS.mac?
      url "https://github.com/brianduff/heimdall/releases/download/v#{version}/heimdall-#{version}.tar.gz"
      sha256 "f484ee736ca210101977a34668c37afbf7d32961a52d38a338b6e260d51e2b05"
  end

  def install
    bin.install "heimdall"
    etc.install "etc/heimdall"
  end
end