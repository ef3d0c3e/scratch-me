<script lang="ts">
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { LuckyLayout } from '#lib/types.ts';
	import Symbol from './Symbol.svelte';

	/** The Lucky Three grid under the foil; winning cells light up once revealed */
	let { layout, revealed }: { layout: LuckyLayout | null; revealed: boolean } = $props();
	const i18n = getI18n();
</script>

<div class="grid" class:revealed>
	{#each { length: 9 }, i (i)}
		{@const s = layout?.cells[i]}
		<div class="cell" class:win={revealed && !!s && s === layout?.win} style:--i={i}>
			{#if s}<Symbol symbol={s} label={i18n.t(`symbol.${s}`)} />{/if}
		</div>
	{/each}
</div>

<style>
	.grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
		padding: 10px;
		height: 100%;
	}
	.cell {
		display: grid;
		place-items: center;
		padding: 14%;
		border-radius: 14px;
		background: var(--paper);
		border: 2px dashed #e9cf9a;
		transition:
			transform 0.3s ease,
			background 0.3s ease;
	}
	.revealed .cell:not(.win) {
		opacity: 0.85;
	}
	.cell.win {
		background: radial-gradient(circle, #fff7d1, #ffd66b);
		border: 2px solid var(--gold-deep);
		animation: pop 0.9s cubic-bezier(0.3, 1.6, 0.5, 1) both;
		animation-delay: calc(var(--i) * 40ms + 150ms);
		box-shadow: 0 0 0 4px rgba(255, 194, 61, 0.35);
	}
	@keyframes pop {
		0% {
			transform: scale(1);
		}
		40% {
			transform: scale(1.18) rotate(-4deg);
		}
		100% {
			transform: scale(1.04);
		}
	}
</style>
