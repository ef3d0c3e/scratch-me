<script lang="ts">
	import type { ToolId } from '#lib/types.ts';
	/** A scratch tool, drawn inline */
	let { tool }: { tool: ToolId } = $props();
</script>

<svg viewBox="0 0 48 48" aria-hidden="true">
	{#if tool === 'coin'}
		<circle cx="24" cy="26" r="16" fill="#b86e00" />
		<circle cx="24" cy="24" r="16" fill="#ffc23d" />
		<circle cx="24" cy="24" r="11" fill="none" stroke="#e08e0b" stroke-width="2.5" />
		<path
			d="M27.5 19a6 6 0 1 0 0 10"
			fill="none"
			stroke="#9a5a00"
			stroke-width="3.5"
			stroke-linecap="round"
		/>
	{:else if tool === 'key'}
		<circle cx="15" cy="15" r="10" fill="none" stroke="#b8c3cf" stroke-width="6" />
		<path
			d="M22 22l19 19M33 33l-4 4M38 38l-4 4"
			stroke="#dfe6ee"
			stroke-width="6"
			stroke-linecap="round"
		/>
		<circle cx="15" cy="15" r="10" fill="none" stroke="#fff" stroke-width="1.5" opacity="0.6" />
	{:else if tool === 'eraser'}
		<g transform="rotate(-40 24 24)">
			<rect x="8" y="16" width="32" height="16" rx="4" fill="#ff9db8" />
			<rect x="8" y="16" width="13" height="16" rx="4" fill="#57b8ff" />
			<rect x="19" y="16" width="3" height="16" fill="#2a1650" opacity="0.2" />
		</g>
	{:else}
		<g transform="rotate(-40 24 24)">
			<rect x="6" y="19" width="26" height="10" rx="4" fill="#2a1650" />
			<rect x="9" y="21" width="6" height="6" rx="2" fill="#ff2b4e" />
			<rect x="32" y="21.5" width="14" height="5" rx="2.5" fill="#ff2b4e" opacity="0.9" />
			<rect x="32" y="23" width="14" height="2" rx="1" fill="#fff" />
		</g>
	{/if}
</svg>

<style>
	svg {
		width: 100%;
		height: 100%;
		display: block;
	}
</style>
