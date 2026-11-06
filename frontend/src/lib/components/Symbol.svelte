<script lang="ts">
	import type { LuckySymbol } from '#lib/types.ts';
	/** One Lucky Three symbol, drawn inline so it looks the same on every phone */
	let { symbol, label }: { symbol: LuckySymbol; label: string } = $props();
</script>

<svg viewBox="0 0 48 48" role="img" aria-label={label}>
	{#if symbol === 'gem'}
		<path d="M14 8h20l10 12-20 22L4 20z" fill="#57b8ff" />
		<path d="M14 8l-10 12h40L34 8z" fill="#9fdcff" />
		<path
			d="M14 8l4 12h12l4-12M4 20l20 22 6-22M18 20l6 22"
			fill="none"
			stroke="#1f5fa8"
			stroke-width="2"
			stroke-linejoin="round"
		/>
		<path d="M11 14l3-3" stroke="#fff" stroke-width="2.5" stroke-linecap="round" />
	{:else if symbol === 'coin'}
		<circle cx="24" cy="25" r="18" fill="#b86e00" />
		<circle cx="24" cy="23" r="18" fill="#ffc23d" />
		<circle cx="24" cy="23" r="13" fill="none" stroke="#e08e0b" stroke-width="2.5" />
		<path
			d="M28.5 17.5a7 7 0 1 0 0 11"
			fill="none"
			stroke="#9a5a00"
			stroke-width="4"
			stroke-linecap="round"
		/>
		<path
			d="M14 13a14 14 0 0 1 8-4"
			stroke="#fff3c4"
			stroke-width="2.5"
			stroke-linecap="round"
			fill="none"
		/>
	{:else if symbol === 'clover'}
		<path
			d="M24 26c2 8 4 12 8 16"
			stroke="#1f7a46"
			stroke-width="3.5"
			stroke-linecap="round"
			fill="none"
		/>
		<g fill="#34e0a1" stroke="#1f7a46" stroke-width="2">
			<circle cx="17" cy="17" r="8" />
			<circle cx="31" cy="17" r="8" />
			<circle cx="17" cy="31" r="8" />
			<circle cx="31" cy="31" r="8" />
		</g>
		<circle cx="24" cy="24" r="4" fill="#1f7a46" />
	{:else if symbol === 'ticket'}
		<g transform="rotate(-12 24 24)">
			<path
				d="M6 14h36v7a3 3 0 0 0 0 6v7H6v-7a3 3 0 0 0 0-6z"
				fill="#ff4f8b"
				stroke="#8a1240"
				stroke-width="2"
				stroke-linejoin="round"
			/>
			<path d="M15 15v18" stroke="#ffd1e1" stroke-width="2" stroke-dasharray="3 3" />
			<path d="M28 18l2 4 4.5.5-3.3 3 1 4.5-4.2-2.4-4.2 2.4 1-4.5-3.3-3 4.5-.5z" fill="#ffd66b" />
		</g>
	{:else if symbol === 'hourglass'}
		<rect x="10" y="5" width="28" height="5" rx="2" fill="#8a5a2b" />
		<rect x="10" y="38" width="28" height="5" rx="2" fill="#8a5a2b" />
		<path
			d="M14 10h20c0 8-8 10-8 14s8 6 8 14H14c0-8 8-10 8-14s-8-6-8-14z"
			fill="#e8f6ff"
			stroke="#57b8ff"
			stroke-width="2"
		/>
		<path d="M18 36c2-4 4-5 6-5s4 1 6 5zM19 14h10c-1 3-3 5-5 6-2-1-4-3-5-6z" fill="#ffc23d" />
	{:else}
		<path
			d="M24 5l5.6 12 13 1.5-9.7 8.8 2.7 12.9L24 33.6l-11.6 6.6 2.7-12.9-9.7-8.8 13-1.5z"
			fill="#b28cff"
			stroke="#5b3aa8"
			stroke-width="2"
			stroke-linejoin="round"
		/>
	{/if}
</svg>

<style>
	svg {
		width: 100%;
		height: 100%;
		display: block;
		filter: drop-shadow(0 2px 0 rgba(42, 22, 80, 0.25));
	}
</style>
