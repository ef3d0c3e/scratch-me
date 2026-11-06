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
	{:else if tool === 'card'}
		<g transform="rotate(-25 24 24)">
			<rect x="6" y="12" width="36" height="24" rx="4" fill="#3651f5" />
			<rect x="6" y="17" width="36" height="5" fill="#1b1530" />
			<rect x="10" y="27" width="10" height="5" rx="1.5" fill="#ffc23d" />
		</g>
	{:else if tool === 'paw'}
		<g fill="#e9c27d" stroke="#8a5a2b" stroke-width="2">
			<ellipse cx="24" cy="31" rx="10" ry="8" />
			<ellipse cx="12" cy="20" rx="4" ry="5" />
			<ellipse cx="20" cy="13" rx="4" ry="5" />
			<ellipse cx="29" cy="13" rx="4" ry="5" />
			<ellipse cx="37" cy="20" rx="4" ry="5" />
		</g>
	{:else if tool === 'wand'}
		<path d="M10 40L30 20" stroke="#2a1650" stroke-width="5" stroke-linecap="round" />
		<path d="M10 40l4-4" stroke="#fff" stroke-width="5" stroke-linecap="round" />
		<path
			d="M34 6l2.6 6 6.4.6-4.9 4.2 1.5 6.3L34 20l-5.6 3.1 1.5-6.3L25 12.6l6.4-.6z"
			fill="#ffd66b"
			stroke="#c77d00"
			stroke-width="1.5"
		/>
	{:else if tool === 'shovel'}
		<g transform="rotate(-35 24 24)">
			<rect x="22" y="2" width="4" height="24" rx="2" fill="#8a5a2b" />
			<rect x="17" y="2" width="14" height="5" rx="2" fill="#8a5a2b" />
			<path
				d="M15 26h18v8c0 6-4 11-9 11s-9-5-9-11z"
				fill="#b8c3cf"
				stroke="#6b7785"
				stroke-width="2"
			/>
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
