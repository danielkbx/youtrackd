cask "ytd" do
  version "{{VERSION}}"

  on_arm do
    url "https://github.com/danielkbx/youtrackd/releases/download/v#{version}/ytd-aarch64-apple-darwin.tar.gz"
    sha256 "{{SHA256_DARWIN_ARM64}}"
  end
  on_intel do
    url "https://github.com/danielkbx/youtrackd/releases/download/v#{version}/ytd-x86_64-apple-darwin.tar.gz"
    sha256 "{{SHA256_DARWIN_AMD64}}"
  end

  name "ytd"
  desc "CLI tool for reading and editing YouTrack tickets and knowledge base articles"
  homepage "https://github.com/danielkbx/youtrackd"

  binary "ytd", quarantine: false
end
