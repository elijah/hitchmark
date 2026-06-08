class Hookmarks < Formula
  desc "Stable, addressable links to documents and paragraphs via hook:// URIs"
  homepage "https://github.com/yourusername/hitchmark"
  url "https://github.com/yourusername/hitchmark/archive/refs/tags/v0.1.0.tar.gz"
  # Update sha256 after publishing the GitHub release:
  #   curl -L <url> | shasum -a 256
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "MIT"
  head "https://github.com/yourusername/hitchmark.git", branch: "master"

  bottle do
    sha256 cellar: :any_skip_relocation, arm64_sonoma:  "0000000000000000000000000000000000000000000000000000000000000000"
    sha256 cellar: :any_skip_relocation, arm64_ventura: "0000000000000000000000000000000000000000000000000000000000000000"
    sha256 cellar: :any_skip_relocation, sonoma:        "0000000000000000000000000000000000000000000000000000000000000000"
    sha256 cellar: :any_skip_relocation, ventura:       "0000000000000000000000000000000000000000000000000000000000000000"
    sha256 cellar: :any_skip_relocation, x86_64_linux:  "0000000000000000000000000000000000000000000000000000000000000000"
  end

  depends_on "rust" => :build

  def install
    # Build only the CLI binary; skip macOS app and Linux daemon
    system "cargo", "build", "--release", "-p", "hitchmark-cli",
           "--locked",
           *std_cargo_args(root: buildpath, path: "crates/hitchmark-cli")

    bin.install "target/release/hk"

    # Shell completions
    generate_completions_from_executable(bin/"hk", "completions")
  end

  def caveats
    <<~EOS
      hookmarks stores links in: ~/.config/hookmarks/store.db

      To start the local HTTP API server (for Obsidian plugin / editor integrations):
        hk serve

      To install the macOS menu bar app, build from source:
        cd #{HOMEBREW_PREFIX}/Cellar/hookmarks/#{version}
        swift build -c release --package-path apps/macos
    EOS
  end

  test do
    # Basic smoke test — hk must start and report its version
    assert_match version.to_s, shell_output("#{bin}/hk --version")

    # hk file must convert a real path to a hook:// URI
    output = shell_output("#{bin}/hk file #{testpath}/test.txt 2>&1", 1)
    # File doesn't exist so it errors, but it must be an hk error not a missing binary
    refute_match "command not found", output

    # Create a real file and convert it
    (testpath/"note.md").write("# Test\nHello world")
    uri_output = shell_output("#{bin}/hk file #{testpath}/note.md").strip
    assert_match "hook://file/", uri_output

    # Round-trip: hk link two URIs
    uri_a = uri_output
    (testpath/"ref.md").write("# Reference")
    uri_b = shell_output("#{bin}/hk file #{testpath}/ref.md").strip
    system bin/"hk", "link", uri_a, uri_b, "--yes"

    # hk list should now show the link
    list_output = shell_output("#{bin}/hk list #{uri_a} --json")
    assert_match uri_b, list_output
  end
end
