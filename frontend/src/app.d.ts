// See https://svelte.dev/docs/kit/types#app.d.ts
import type { Lang } from '#lib/i18n/index.svelte.ts';
import type { ThemeId } from '#lib/types.ts';

declare global {
	namespace App {
		interface Locals {
			lang: Lang;
			skin: ThemeId;
		}
	}
}

export {};
