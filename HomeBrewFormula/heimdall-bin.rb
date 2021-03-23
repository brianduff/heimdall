class HeimdallBin < Formula
  version '0.5'
  desc "Guard the Bifrost."
  homepage "https://github.com/brianduff/heimdall"

  if OS.mac?
      url "https://github.com/brianduff/heimdall/releases/download/v#{version}/heimdall-#{version}.tar.gz"
      sha256 "df9954b321aca970d22cd4b2af01a0e6d4f14e3ba66b855946a3af386fabaa87"
  end

  def install
    bin.install "heimdall"
    etc.install "etc/heimdall"
  end
end