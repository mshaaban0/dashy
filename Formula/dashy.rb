# Homebrew formula for Dashy
# To use: brew tap mshaaban0/dashy && brew install dashy

class Dashy < Formula
  desc "Fast, lightweight terminal system monitor"
  homepage "https://github.com/mshaaban0/dashy"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/mshaaban0/dashy/releases/download/v#{version}/dashy-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM_MACOS"
    end
    on_intel do
      url "https://github.com/mshaaban0/dashy/releases/download/v#{version}/dashy-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_INTEL_MACOS"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/mshaaban0/dashy/releases/download/v#{version}/dashy-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM_LINUX"
    end
    on_intel do
      url "https://github.com/mshaaban0/dashy/releases/download/v#{version}/dashy-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "PLACEHOLDER_SHA256_INTEL_LINUX"
    end
  end

  def install
    bin.install "dashy"
  end

  test do
    assert_match "dashy", shell_output("#{bin}/dashy --version 2>&1", 1)
  end
end
