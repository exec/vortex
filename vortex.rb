class Vortex < Formula
  desc "Lightning-fast ephemeral VM platform with hardware-level isolation"
  homepage "https://github.com/exec/vortex"
  version "0.3.0"
  
  if Hardware::CPU.arm?
    url "https://github.com/exec/vortex/releases/download/v0.3.0/vortex-v0.3.0-aarch64-apple-darwin.tar.gz"
    sha256 "PLACEHOLDER_ARM64_SHA256"
  else
    url "https://github.com/exec/vortex/releases/download/v0.3.0/vortex-v0.3.0-x86_64-apple-darwin.tar.gz"  
    sha256 "PLACEHOLDER_AMD64_SHA256"
  end
  
  license "MIT"
  
  depends_on "krunvm"
  
  def install
    bin.install "bin/vortex"
  end
  
  test do
    system "#{bin}/vortex", "--version"
    system "#{bin}/vortex", "--help"
  end
end