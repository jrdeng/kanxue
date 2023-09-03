#include <Windows.h>
#include <stdint.h>
#include <stdio.h>
#include <stdbool.h>

#pragma comment(lib, "User32.lib")

void c_raise_privilege()
{
    // to read memory of some processes, we need this
    HANDLE hToken;
    TOKEN_PRIVILEGES tp;
    tp.PrivilegeCount = 1;
    tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
    if (OpenProcessToken(GetCurrentProcess(), TOKEN_ALL_ACCESS, &hToken))
    {
        if (LookupPrivilegeValue(NULL, SE_DEBUG_NAME, &tp.Privileges[0].Luid))
        {
            AdjustTokenPrivileges(hToken, FALSE, &tp, 0, NULL, 0);
        }
    }
    if (hToken)
    {
        CloseHandle(hToken);
    }
}

int64_t c_window_at_cursor_point()
{
    POINT winPos;
    if (GetCursorPos(&winPos))
    {
        return (int64_t)WindowFromPoint(winPos);
    }
    return 0;
}

int64_t c_open_process(int64_t hwnd)
{
    DWORD pid = 0;
    GetWindowThreadProcessId((HWND)hwnd, &pid);
    int64_t handle = OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid);
    return handle;
}

void c_close_handle(int64_t handle)
{
    CloseHandle((HANDLE)handle);
}

bool c_read_memory(int64_t handle, int64_t address, void *data, size_t len)
{
    return ReadProcessMemory((HANDLE)handle, (LPCVOID)address, data, len, NULL) != 0;
}

const char *c_read_memory_as_string(int64_t handle, int64_t address, size_t len)
{
    // printf("reading memory as string at %p in handle(%ld)\n", (void*)address, handle);
    char *buffer = (char *)malloc(len + 1);
    memset(buffer, 0, len + 1);
    if (c_read_memory(handle, address, buffer, len))
    {
        // printf("got mem as string: %s\n", buffer);
        return buffer;
    }
    else
    {
        // printf("failed to read memory as string\n");
        free(buffer);
        return NULL;
    }
}

void c_free_string(const char *ptr)
{
    if (ptr != NULL)
    {
        free(ptr);
    }
}
