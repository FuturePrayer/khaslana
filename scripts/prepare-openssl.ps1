# 为 Windows x64 MSVC 准备预编译 OpenSSL，不安装软件、不修改系统环境。
# 发布来源：https://github.com/TaurusTLS-Developers/OpenSSL-Distribution/releases/tag/v3.5.9
# 该包的 lib/static 使用静态 CRT（LIBCMT）；不要误用 lib/import 的 DLL 导入库。
$ErrorActionPreference = 'Stop'

$version = '3.5.9'
$expectedHash = '1F4D8E4221A40B5DE2CF91D45CCC730AACF3D945BC1E4BC4532F421AEA6B3680'
$repoRoot = Split-Path -Parent $PSScriptRoot
$downloadDir = Join-Path $repoRoot 'target/deps/downloads'
$archiveName = "openssl-$version-Windows-x64.zip"
$archivePath = Join-Path $downloadDir $archiveName
$opensslRoot = Join-Path $repoRoot "target/deps/openssl-$version-windows-x64"
$downloadUrl = "https://github.com/TaurusTLS-Developers/OpenSSL-Distribution/releases/download/v$version/$archiveName"

New-Item -ItemType Directory -Path $downloadDir -Force | Out-Null
if (-not (Test-Path -LiteralPath $archivePath -PathType Leaf)) {
    $partialPath = "$archivePath.partial"
    Invoke-WebRequest -Uri $downloadUrl -OutFile $partialPath
    $actualHash = (Get-FileHash -LiteralPath $partialPath -Algorithm SHA256).Hash
    if ($actualHash -ne $expectedHash) {
        throw "OpenSSL 下载校验失败：期望 $expectedHash，实际 $actualHash"
    }
    Move-Item -LiteralPath $partialPath -Destination $archivePath -Force
}

# 缓存也校验；失败必须显式报错，不能继续使用未知库或回退源码编译。
$actualHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash
if ($actualHash -ne $expectedHash) {
    throw "OpenSSL 缓存校验失败，请移除该压缩包后重试：$archivePath"
}
Expand-Archive -LiteralPath $archivePath -DestinationPath $opensslRoot -Force
foreach ($relativePath in @('include/openssl/opensslv.h', 'lib/static/libssl.lib', 'lib/static/libcrypto.lib', 'LICENSE.txt')) {
    if (-not (Test-Path -LiteralPath (Join-Path $opensslRoot $relativePath) -PathType Leaf)) {
        throw "OpenSSL 预编译包缺少文件：$relativePath"
    }
}
$versionHeader = Get-Content -LiteralPath (Join-Path $opensslRoot 'include/openssl/opensslv.h') -Raw
if ($versionHeader -notmatch ('#\s*define\s+OPENSSL_VERSION_STR\s+"' + [regex]::Escape($version) + '"')) {
    throw 'OpenSSL 头文件版本与固定版本不一致'
}
Write-Output "OpenSSL $version 预编译静态库已就绪：$opensslRoot"

