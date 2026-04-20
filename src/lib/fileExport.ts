interface PickerAcceptType {
  description?: string;
  accept: Record<string, string[]>;
}

interface SaveFilePickerOptionsLike {
  suggestedName?: string;
  types?: PickerAcceptType[];
  excludeAcceptAllOption?: boolean;
}

interface OpenFilePickerOptionsLike {
  multiple?: boolean;
  types?: PickerAcceptType[];
  excludeAcceptAllOption?: boolean;
}

interface FileSystemWritableFileStreamLike {
  write(data: string): Promise<void>;
  close(): Promise<void>;
}

interface SaveFileHandleLike {
  createWritable(): Promise<FileSystemWritableFileStreamLike>;
}

interface OpenFileHandleLike {
  getFile(): Promise<File>;
}

type SavePickerWindow = Window & {
  showSaveFilePicker?: (options?: SaveFilePickerOptionsLike) => Promise<SaveFileHandleLike>;
};

type OpenPickerWindow = Window & {
  showOpenFilePicker?: (options?: OpenFilePickerOptionsLike) => Promise<OpenFileHandleLike[]>;
};

export function downloadTextFile(filename: string, content: string, contentType = "application/json;charset=utf-8") {
  const blob = new Blob([content], { type: contentType });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.style.display = "none";
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function isAbortError(error: unknown) {
  return error instanceof DOMException && error.name === "AbortError";
}

export async function saveTextFile(
  filename: string,
  content: string,
  contentType = "application/json;charset=utf-8",
): Promise<boolean> {
  const pickerContentType = contentType.split(";")[0] || "application/json";
  const pickerWindow = window as SavePickerWindow;
  if (typeof pickerWindow.showSaveFilePicker === "function") {
    try {
      const handle = await pickerWindow.showSaveFilePicker({
        suggestedName: filename,
        excludeAcceptAllOption: false,
        types: [
          {
            description: "JSON files",
            accept: {
              [pickerContentType]: [".json"],
            },
          },
        ],
      });
      const writable = await handle.createWritable();
      await writable.write(content);
      await writable.close();
      return true;
    } catch (error) {
      if (isAbortError(error)) {
        return false;
      }
      throw error;
    }
  }

  downloadTextFile(filename, content, contentType);
  return true;
}

export async function openTextFile(): Promise<string | null> {
  const pickerWindow = window as OpenPickerWindow;
  if (typeof pickerWindow.showOpenFilePicker === "function") {
    try {
      const [handle] = await pickerWindow.showOpenFilePicker({
        multiple: false,
        excludeAcceptAllOption: false,
        types: [
          {
            description: "JSON files",
            accept: {
              "application/json": [".json"],
            },
          },
        ],
      });
      if (!handle) {
        return null;
      }
      const file = await handle.getFile();
      return await file.text();
    } catch (error) {
      if (isAbortError(error)) {
        return null;
      }
      throw error;
    }
  }

  return await new Promise<string | null>((resolve, reject) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json,application/json";
    input.style.display = "none";
    document.body.appendChild(input);

    input.addEventListener("change", () => {
      const file = input.files?.[0];
      if (!file) {
        input.remove();
        resolve(null);
        return;
      }
      void file
        .text()
        .then((text) => resolve(text))
        .catch(reject)
        .finally(() => input.remove());
    });

    input.click();
  });
}
