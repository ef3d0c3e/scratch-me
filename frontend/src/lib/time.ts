/** "1:05:09", "4:07", "0:09": time left until `at`, never negative */
export function countdown(at: string | Date, now: number): string {
	const ms = Math.max(0, new Date(at).getTime() - now);
	const s = Math.ceil(ms / 1000);
	const h = Math.floor(s / 3600);
	const m = Math.floor((s % 3600) / 60);
	const sec = String(s % 60).padStart(2, '0');
	return h > 0 ? `${h}:${String(m).padStart(2, '0')}:${sec}` : `${m}:${sec}`;
}

/** Whether `at` has come */
export const due = (at: string | Date | null, now: number) =>
	at !== null && new Date(at).getTime() <= now;
