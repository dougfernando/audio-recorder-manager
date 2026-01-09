import { writable, derived } from "svelte/store";

// Recording state
export const isRecording = writable(false);
export const currentSession = writable(null);
export const recordingStatus = writable(null);
export const activeSessions = writable([]);

// Recordings list
export const recordings = writable([]);
export const recoveryList = writable([]);

// Device status
export const devices = writable([]);

// UI state
export const selectedDuration = writable(60);
export const selectedFormat = writable("m4a");
export const selectedQuality = writable("professional");
export const isManualMode = writable(true);

// Derived stores
export const durationInSeconds = derived(
    [selectedDuration, isManualMode],
    ([$selectedDuration, $isManualMode]) => {
        return $isManualMode ? -1 : $selectedDuration;
    },
);

// Format recording duration display
export function formatDuration(seconds) {
    if (seconds < 0) return "Manual";
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    if (remainingSeconds === 0) return `${minutes}m`;
    return `${minutes}m ${remainingSeconds}s`;
}

// Format file size
export function formatFileSize(bytes) {
    if (bytes === 0) return "0 Bytes";
    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
}

// Format elapsed time
export function formatTime(seconds) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
        return `${hours}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
    }
    return `${minutes}:${secs.toString().padStart(2, "0")}`;
}
