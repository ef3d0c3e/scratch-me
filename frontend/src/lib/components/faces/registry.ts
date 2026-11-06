/**
 * Which face draws which series. Scratch faces get the layout once scratched
 * and the foil to put over it; played faces run their own moves against the
 * server and hand back the end.
 */

import type { Component } from 'svelte';
import type { Series } from '#lib/types.ts';
import BingoFace from './BingoFace.svelte';
import BoxesFace from './BoxesFace.svelte';
import BombFace from './BombFace.svelte';
import ChainFace from './ChainFace.svelte';
import DerbyFace from './DerbyFace.svelte';
import DiceFace from './DiceFace.svelte';
import GoldenFace from './GoldenFace.svelte';
import HighCardFace from './HighCardFace.svelte';
import LuckyFace from './LuckyFace.svelte';
import NumberFace from './NumberFace.svelte';
import PlinkoFace from './PlinkoFace.svelte';
import PushLuckFace from './PushLuckFace.svelte';
import ReelsFace from './ReelsFace.svelte';
import RocketFace from './RocketFace.svelte';
import SafeFace from './SafeFace.svelte';
import TicTacToeFace from './TicTacToeFace.svelte';
import TreasureFace from './TreasureFace.svelte';
import WheelFace from './WheelFace.svelte';

// Each face types its own layout: the registry only knows they are faces
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Face = Component<any>;

export const SCRATCH_FACES: Partial<Record<Series, Face>> = {
	lucky_three: LuckyFace,
	number_match: NumberFace,
	ticket_chain: ChainFace,
	cash_wheel: WheelFace,
	golden: GoldenFace,
	high_card: HighCardFace,
	lucky_reels: ReelsFace,
	bingo: BingoFace,
	tic_tac_toe: TicTacToeFace,
	lucky_dice: DiceFace,
	derby: DerbyFace,
	plinko: PlinkoFace
};

export const PLAYED_FACES: Partial<Record<Series, Face>> = {
	treasure_dig: TreasureFace,
	push_luck: PushLuckFace,
	safe_cracker: SafeFace,
	mystery_boxes: BoxesFace,
	defuse_bomb: BombFace,
	rocket: RocketFace
};
