class Ytd < Formula
  desc "CLI tool for reading and editing YouTrack tickets and knowledge base articles"
  homepage "https://github.com/danielkbx/youtrackd"
  version "{{VERSION}}"
  license "MIT"
  bottle :unneeded

  on_macos do
    on_arm do
      url "https://github.com/danielkbx/youtrackd/releases/download/v{{VERSION}}/ytd-aarch64-apple-darwin.tar.gz"
      sha256 "{{SHA256_DARWIN_ARM64}}"
    end
    on_intel do
      url "https://github.com/danielkbx/youtrackd/releases/download/v{{VERSION}}/ytd-x86_64-apple-darwin.tar.gz"
      sha256 "{{SHA256_DARWIN_AMD64}}"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/danielkbx/youtrackd/releases/download/v{{VERSION}}/ytd-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "{{SHA256_LINUX_ARM64}}"
    end
    on_intel do
      url "https://github.com/danielkbx/youtrackd/releases/download/v{{VERSION}}/ytd-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "{{SHA256_LINUX_AMD64}}"
    end
  end

  def install
    bin.install "ytd"
  end

  test do
    assert_match "ytd", shell_output("#{bin}/ytd help")
  end
end
