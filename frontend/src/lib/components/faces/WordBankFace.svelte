<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { WordBankLayout } from '#lib/types.ts';

	/** The words are in view; your letters are under the foil */
	interface Props {
		layout: WordBankLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});
	const mine = (ch: string) => revealed && !!layout?.letters.includes(ch);
</script>

<p class="rules">{i18n.t('series.word_bank.rules')}</p>
<ul class="words">
	{#each layout?.words ?? [] as w, i (i)}
		<li
			class:done={revealed && layout?.complete.includes(i)}
			data-done={revealed && layout?.complete.includes(i) ? '' : undefined}
		>
			{#each w.split('') as ch, k (k)}<span class:got={mine(ch)}>{ch}</span>{/each}
		</li>
	{:else}
		{#each { length: 5 }, i (i)}<li class="blank">• • • • •</li>{/each}
	{/each}
</ul>
<p class="label">{i18n.t('words.yours')}</p>
<div class="play strip">
	<div class="letters">
		{#each { length: 12 }, i (i)}
			<span class="letter">{layout?.letters[i] ?? ''}</span>
		{/each}
	</div>
	{@render foil()}
</div>
<p class="pays">{i18n.t('words.pays')}</p>

<style>
	.words {
		list-style: none;
		margin: 0 0 10px;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
		align-items: center;
	}
	.words li {
		display: flex;
		gap: 3px;
	}
	.words span {
		width: 30px;
		height: 32px;
		display: grid;
		place-items: center;
		border-radius: 6px;
		background: var(--paper);
		border: 1.5px solid #a3e635;
		font: 400 18px/1 var(--display);
	}
	.words span.got {
		background: #d9f99d;
	}
	.words li.done span {
		background: #84cc16;
		color: #fff;
		border-color: #4d7c0f;
		animation: pop 0.5s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	.blank {
		opacity: 0.4;
	}
	.label {
		text-align: center;
		font-size: 11px;
		font-weight: 800;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		opacity: 0.7;
		margin-bottom: 6px;
	}
	.strip {
		aspect-ratio: 6 / 2.1 !important;
	}
	.letters {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(6, 1fr);
		gap: 5px;
		padding: 8px;
	}
	.letter {
		display: grid;
		place-items: center;
		border-radius: 8px;
		background: var(--paper);
		font: 400 22px/1 var(--display);
		color: #4d7c0f;
	}
	.pays {
		text-align: center;
		font-size: 12px;
		font-weight: 700;
		margin-top: 8px;
	}
	@keyframes pop {
		40% {
			transform: scale(1.15);
		}
	}
</style>
