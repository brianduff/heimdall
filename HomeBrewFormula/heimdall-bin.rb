class HeimdallBin < Formula
  version '0.1'
  desc "Guard the Bifrost."
  homepage "https://github.com/brianduff/heimdall"

  if OS.mac?
      url "https://github.com/BurntSushi/ripgrep/releases/download/#{version}/heimdall-#{version}.tar.gz"
      sha256 "a8214234a2ae96ff599942e54bcf0cc01fd15ac947512a98be7e8247dd917d8b"
  end

  def install
    bin.install "heimdall"
  end
end