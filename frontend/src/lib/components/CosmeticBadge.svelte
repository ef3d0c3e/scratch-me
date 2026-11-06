<script lang="ts">
	import type { Cosmetic } from '#lib/types.ts';
	import { SKIN_SWATCH } from '#lib/series.ts';
	import ToolIcon from './ToolIcon.svelte';

	/** A theme as a swatch of its colours, a tool as its icon */
	let { item, size = 'small' }: { item: Cosmetic; size?: 'small' | 'big' } = $props();
</script>

<span class="badge {size}" data-item="{item.kind}:{item.id}">
	{#if item.kind === 'theme'}
		{@const c = SKIN_SWATCH[item.id]}
		<span
			class="swatch"
			style:background="conic-gradient({c[0]} 0 25%, {c[1]} 0 50%, {c[2]} 0 75%, {c[3]} 0)"
		></span>
	{:else}
		<ToolIcon tool={item.id} />
	{/if}
</span>

<style>
	.badge {
		display: inline-grid;
		place-items: center;
		flex: none;
	}
	.small {
		width: 40px;
		height: 40px;
	}
	.big {
		width: 96px;
		height: 96px;
	}
	.swatch {
		width: 86%;
		height: 86%;
		border-radius: 50%;
		box-shadow:
			0 0 0 3px rgba(255, 255, 255, 0.85),
			0 4px 12px rgba(0, 0, 0, 0.35);
	}
</style>
