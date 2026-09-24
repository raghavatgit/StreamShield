// StreamShield Memory Integrity: Text Section Hash Verification
// Computes cryptographic hash of .text section to identify unauthorized memory modifications.

#include <windows.h>
#include <wincrypt.h>
#include <vector>
#include <iostream>

bool ComputeTextSectionSHA256(HMODULE hMod, std::vector<BYTE>& hashOut) {
    if (!hMod) hMod = GetModuleHandle(NULL);
    PIMAGE_DOS_HEADER dosHeader = reinterpret_cast<PIMAGE_DOS_HEADER>(hMod);
    if (dosHeader->e_magic != IMAGE_DOS_SIGNATURE) return false;

    PIMAGE_NT_HEADERS ntHeaders = reinterpret_cast<PIMAGE_NT_HEADERS>(
        reinterpret_cast<BYTE*>(hMod) + dosHeader->e_lfanew
    );
    if (ntHeaders->Signature != IMAGE_NT_SIGNATURE) return false;

    PIMAGE_SECTION_HEADER section = IMAGE_FIRST_SECTION(ntHeaders);
    PIMAGE_SECTION_HEADER textSection = nullptr;

    for (WORD i = 0; i < ntHeaders->FileHeader.NumberOfSections; ++i) {
        if (strncmp(reinterpret_cast<const char*>(section[i].Name), ".text", 5) == 0) {
            textSection = &section[i];
            break;
        }
    }

    if (!textSection) return false;

    BYTE* pSectionData = reinterpret_cast<BYTE*>(hMod) + textSection->VirtualAddress;
    DWORD sectionSize = textSection->Misc.VirtualSize;

    HCRYPTPROV hProv = 0;
    HCRYPTHASH hHash = 0;
    if (!CryptAcquireContext(&hProv, NULL, NULL, PROV_RSA_AES, CRYPT_VERIFYCONTEXT)) return false;
    if (!CryptCreateHash(hProv, CALG_SHA_256, 0, 0, &hHash)) {
        CryptReleaseContext(hProv, 0);
        return false;
    }

    if (!CryptHashData(hHash, pSectionData, sectionSize, 0)) {
        CryptDestroyHash(hHash);
        CryptReleaseContext(hProv, 0);
        return false;
    }

    DWORD hashLen = 32;
    hashOut.resize(hashLen);
    CryptGetHashParam(hHash, HP_HASHVAL, hashOut.data(), &hashLen, 0);

    CryptDestroyHash(hHash);
    CryptReleaseContext(hProv, 0);
    return true;
}
