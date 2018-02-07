#ifdef _WIN32
#include <windows.h>
#include <tlhelp32.h>

int getppid(void)
{
	DWORD ppid = -1;

	HANDLE snapshot;
	PROCESSENTRY32 pe32;

	snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
	if (snapshot == INVALID_HANDLE_VALUE)
		return ppid;

	pe32.dwSize = sizeof pe32;
	if (!Process32First(snapshot, &pe32))
		return ppid;

	DWORD pid = GetCurrentProcessId();

	do {
		if (pe32.th32ProcessID == pid) {
			ppid = pe32.th32ParentProcessID;
			break;
		}
	} while (Process32Next(snapshot, &pe32));

	CloseHandle(snapshot);

	return ppid;
}
#else
#include <unistd.h>
#endif
