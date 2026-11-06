<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getI18n } from '#lib/i18n/index.svelte.ts';
	import type { TicTacToeLayout } from '#lib/types.ts';

	interface Props {
		layout: TicTacToeLayout | null;
		revealed: boolean;
		foil: Snippet;
		onsettled: () => void;
	}
	let { layout, revealed, foil, onsettled }: Props = $props();
	const i18n = getI18n();
	$effect(() => {
		if (revealed && layout) onsettled();
	});
	const LINES = [
		[0, 1, 2],
		[3, 4, 5],
		[6, 7, 8],
		[0, 3, 6],
		[1, 4, 7],
		[2, 5, 8],
		[0, 4, 8],
		[2, 4, 6]
	];
	/** The cells of X's winning lines on a board */
	const winning = (board: ('x' | 'o')[]) =>
		new Set(LINES.filter((l) => l.every((i) => board[i] === 'x')).flat());
</script>

<p class="rules">{i18n.t('series.tic_tac_toe.rules')}</p>
<div class="play wide">
	<div class="games">
		{#each { length: 3 }, g (g)}
			{@const game = layout?.games[g]}
			{@const lit = game && revealed ? winning(game.board) : new Set<number>()}
			<div
				class="game"
				class:won={revealed && game?.won}
				data-won={revealed && game?.won ? '' : undefined}
			>
				<span class="label">{i18n.t('ttt.game', { n: g + 1 })}</span>
				<div class="board">
					{#each { length: 9 }, i (i)}
						<span class="cell {game?.board[i] ?? ''}" class:lit={lit.has(i)}
							>{game ? game.board[i].toUpperCase() : ''}</span
						>
					{/each}
				</div>
				<b class="coins">{game ? i18n.num(game.coins) : ''}</b>
			</div>
		{/each}
	</div>
	{@render foil()}
</div>

<style>
	.games {
		height: 100%;
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
		padding: 8px;
	}
	.game {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 6px;
		border-radius: 12px;
		background: var(--paper);
	}
	.game.won {
		background: #e0f2fe;
		box-shadow: inset 0 0 0 2px #0369a1;
	}
	.label {
		font-size: 10px;
		font-weight: 800;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		opacity: 0.7;
	}
	.board {
		flex: 1;
		width: 100%;
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 2px;
		background: #0369a1;
		border-radius: 6px;
		overflow: hidden;
	}
	.cell {
		display: grid;
		place-items: center;
		background: var(--paper);
		font: 400 18px/1 var(--display);
	}
	.cell.x {
		color: #0369a1;
	}
	.cell.o {
		color: #e0243c;
	}
	.cell.lit {
		background: #ffd66b;
		animation: pop 0.6s cubic-bezier(0.3, 1.6, 0.5, 1) both;
	}
	@keyframes pop {
		40% {
			transform: scale(1.25);
		}
	}
	.coins {
		font-size: 13px;
		color: #0369a1;
	}
</style>
