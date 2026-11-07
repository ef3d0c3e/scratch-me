/**
 * Which face draws which series. Scratch faces get the layout once scratched
 * and the foil to put over it; played faces run their own moves against the
 * server and hand back the end.
 */

import type { Component } from 'svelte';
import type { Series } from '#lib/types.ts';
import BattleshipFace from './BattleshipFace.svelte';
import CoinFlipFace from './CoinFlipFace.svelte';
import PokerFace from './PokerFace.svelte';
import RouletteFace from './RouletteFace.svelte';
import BingoFace from './BingoFace.svelte';
import BlackjackFace from './BlackjackFace.svelte';
import BoardWalkFace from './BoardWalkFace.svelte';
import FishingFace from './FishingFace.svelte';
import HotSpotFace from './HotSpotFace.svelte';
import WordBankFace from './WordBankFace.svelte';
import WordSearchFace from './WordSearchFace.svelte';
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
import PumpkinFace from './PumpkinFace.svelte';
import SnowballFace from './SnowballFace.svelte';
import PotFace from './PotFace.svelte';
import DuelFace from './DuelFace.svelte';
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
	plinko: PlinkoFace,
	blackjack: BlackjackFace,
	hot_spot: HotSpotFace,
	poker_hand: PokerFace,
	mega_reels: ReelsFace,
	platinum_three: LuckyFace,
	board_walk: BoardWalkFace,
	fishing: FishingFace,
	word_bank: WordBankFace,
	word_search: WordSearchFace,
	pumpkin_patch: PumpkinFace,
	snowball: SnowballFace,
	pot_ticket: PotFace,
	duel: DuelFace
};

export const PLAYED_FACES: Partial<Record<Series, Face>> = {
	treasure_dig: TreasureFace,
	push_luck: PushLuckFace,
	safe_cracker: SafeFace,
	mystery_boxes: BoxesFace,
	defuse_bomb: BombFace,
	rocket: RocketFace,
	battleship: BattleshipFace,
	coin_flip: CoinFlipFace,
	roulette: RouletteFace
};
