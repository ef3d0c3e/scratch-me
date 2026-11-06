/** Shapes of the API's responses, see backend/src/routes */

export type Series =
	| 'lucky_three'
	| 'high_card'
	| 'lucky_reels'
	| 'bingo'
	| 'number_match'
	| 'cash_wheel'
	| 'treasure_dig'
	| 'tic_tac_toe'
	| 'lucky_dice'
	| 'derby'
	| 'plinko'
	| 'push_luck'
	| 'safe_cracker'
	| 'mystery_boxes'
	| 'defuse_bomb'
	| 'rocket'
	| 'blackjack'
	| 'hot_spot'
	| 'battleship'
	| 'board_walk'
	| 'fishing'
	| 'word_bank'
	| 'word_search'
	| 'pumpkin_patch'
	| 'snowball'
	| 'pot_ticket'
	| 'duel'
	| 'ticket_chain'
	| 'golden';
export type Source = 'pocket' | 'envelope' | 'prize' | 'shop' | 'reward' | 'gift' | 'duel';

export type ThemeId =
	| 'midnight'
	| 'sunset'
	| 'mint'
	| 'ocean'
	| 'candy'
	| 'arcade'
	| 'neon'
	| 'royal'
	| 'forest'
	| 'champion'
	| 'spooky'
	| 'frost';
export type ToolId = 'coin' | 'card' | 'key' | 'eraser' | 'paw' | 'laser' | 'wand' | 'shovel';
export type FoilId = 'silver' | 'carbon' | 'leopard' | 'gold' | 'holo' | 'galaxy';
export type Cosmetic =
	{ kind: 'theme'; id: ThemeId } | { kind: 'tool'; id: ToolId } | { kind: 'foil'; id: FoilId };

/** Where a cosmetic comes from */
export type Origin =
	| { kind: 'default' }
	| { kind: 'shop'; price: number }
	| { kind: 'golden' }
	| { kind: 'trophy'; achievement: string };

export interface TicketDto {
	id: number;
	series: Series;
	source: Source;
	created_at: string;
	/** Who sent it, for a gift */
	from?: string | null;
}

export interface Pocket {
	count: number;
	size: number;
	/** When the next ticket comes, null while full */
	next_at: string | null;
}

export interface Contents {
	coins: number;
	tickets: Series[];
}

export interface EnvelopeDto {
	available: boolean;
	streak: number;
	best: number;
	day: number;
	contents: Contents;
	next_at: string;
}

export interface MeDto {
	username: string;
	coins: number;
	pocket: Pocket;
	envelope: EnvelopeDto;
	theme: ThemeId;
	tool: ToolId;
	foil: FoilId;
	/** Cosmetics owned besides the defaults */
	owned: Cosmetic[];
	level: LevelDto;
	/** The achievement shown next to their name */
	title: string | null;
	is_admin: boolean;
	/** Playing without an account: no chat, and signing up keeps it all */
	guest: boolean;
	/** What a pocket ticket is worth at most, at the player's level */
	pocket_worth: number;
	tickets: TicketDto[];
}

export interface LevelDto {
	level: number;
	xp: number;
	/** Where this level started and the next begins */
	from: number;
	to: number;
}

export interface Reward {
	/** A level reached, a quest key or an achievement id */
	what: string;
	reward: Prize;
}

/** What an action's progress gave */
export interface Progress {
	xp_gained: number;
	level: LevelDto;
	level_ups: Reward[];
	quests: Reward[];
	bonus: Prize | null;
	achievements: Reward[];
	tickets: TicketDto[];
	coins: number;
}

export interface QuestDto {
	key: string;
	goal: number;
	progress: number;
	done: boolean;
	reward: Prize;
}

export interface QuestsDto {
	items: QuestDto[];
	bonus: Prize;
	bonus_done: boolean;
	rerolled: boolean;
	resets_at: string;
}

export interface AchievementDto {
	id: string;
	reward: Prize;
	earned_at: string | null;
	progress: [number, number] | null;
}

export type Prize =
	| { kind: 'nothing' }
	| { kind: 'coins'; amount: number }
	| { kind: 'ticket'; series: Series }
	| { kind: 'tickets'; series: Series[] }
	| { kind: 'pocket_refill' }
	| { kind: 'cosmetic'; item: Cosmetic }
	| { kind: 'bundle'; items: Prize[] };

/** A ticket's bonus box: its coins multiplied, or one more ticket */
export type Bonus = { kind: 'times'; times: number } | { kind: 'ticket'; series: Series };

/** A ticket played move by move: what may be seen, and the end when it comes */
export interface PlayState<V = Record<string, unknown>> {
	view: V;
	done: Scratched | null;
}

export type LuckySymbol = 'gem' | 'coin' | 'clover' | 'ticket' | 'hourglass' | 'star';

export interface LuckyLayout {
	cells: LuckySymbol[];
	win: LuckySymbol | null;
}

export interface NumberLayout {
	winning: number[];
	spots: { n: number; coins: number; hit: boolean }[];
}

export interface ChainLayout {
	panels: (Series | null)[];
}

export interface WheelLayout {
	base: number;
	wheel: number[];
	/** The segment the wheel stops on */
	stop: number;
}

export interface GoldenLayout {
	prize: Prize;
}

export interface Scratched {
	id: number;
	series: Series;
	layout: unknown;
	prize: Prize;
	coins: number;
	/** Tickets won */
	tickets: TicketDto[];
	/** The cosmetic won was already owned: coins instead */
	duplicate: boolean;
	progress: Progress;
}

export interface ShopDto {
	/** `until`: a seasonal series' last day on sale, as [month, day] */
	tickets: { series: Series; price: number; level: number; until: [number, number] | null }[];
	cosmetics: { item: Cosmetic; origin: Origin }[];
}

export interface Bought {
	coins: number;
	ticket: TicketDto | null;
	/** A gift went to this player */
	gift_to?: string | null;
	item: Cosmetic | null;
	progress: Progress;
}

export interface PocketClaim {
	tickets: TicketDto[];
	pocket: Pocket;
	progress: Progress;
}

export interface EnvelopeClaim {
	day: number;
	contents: Contents;
	coins: number;
	tickets: TicketDto[];
	progress: Progress;
}

export interface ChatEvent {
	id: number;
	user: string;
	level: number;
	title: string | null;
	body: string;
	at: string;
}

export interface WinEvent {
	user: string;
	series: Series;
	prize: Prize;
	at: string;
}

export interface FeatEvent {
	user: string;
	/** The achievement earned */
	id: string;
	at: string;
}

export type FeedItem = ({ type: 'win' } & WinEvent) | ({ type: 'feat' } & FeatEvent);

export interface AnnouncementDto {
	id: number;
	body: string;
	at: string;
}

export interface Shouted {
	shout: ChatEvent;
	progress: Progress;
}

export interface ProfileDto {
	username: string;
	/** No gifts, duels or chat with a guest */
	guest: boolean;
	level: LevelDto;
	title: string | null;
	theme: ThemeId;
	joined: string;
	online: boolean;
	scratched: number;
	wins: number;
	best_win: number;
	best_streak: number;
	/** Achievements earned, newest first */
	achievements: string[];
	/** The viewer muted them */
	muted: boolean;
}

export interface PlayingCard {
	/** 2 to 14 (the ace) */
	rank: number;
	suit: 'spades' | 'hearts' | 'diamonds' | 'clubs';
}

export interface HighCardLayout {
	dealer: PlayingCard;
	hands: { card: PlayingCard; coins: number }[];
}

export type ReelSymbol = 'cherry' | 'lemon' | 'bell' | 'bar' | 'seven';

export interface ReelsLayout {
	/** Reel by reel, top to bottom */
	reels: ReelSymbol[][];
	/** Paylines that pay: rows 0–2, then the two diagonals */
	lines: number[];
}

export interface BingoLayout {
	/** Row by row; 0 is the free middle */
	card: number[][];
	calls: number[];
	lines: number[];
}

export type Spot = { kind: 'sand' } | { kind: 'chest'; coins: number } | { kind: 'map' };

export interface DigState {
	dug: { cell: number; spot: Spot }[];
	digs_left: number;
	/** After the last dig: the whole beach and the prize */
	done: Scratched | null;
}

export interface TicTacToeLayout {
	games: { board: ('x' | 'o')[]; coins: number; won: boolean }[];
}

export interface DiceLayout {
	dice: number[];
	roll: 'sixes' | 'triple' | 'straight' | 'pair' | 'nothing';
}

export interface DerbyLayout {
	bets: { horse: number; coins: number }[];
	/** Horses in finishing order: the first one wins */
	order: number[];
}

export interface PlinkoLayout {
	balls: { path: boolean[]; bin: number }[];
	bins: number[];
}

export interface BlackjackLayout {
	dealer: PlayingCard[];
	hands: { cards: PlayingCard[]; coins: number; result: 'blackjack' | 'win' | 'push' | 'lose' }[];
}

export interface HotSpotLayout {
	mine: number[];
	drawn: number[];
	matches: number;
}

export interface BattleshipLayout {
	ships: { cells: number[]; coins: number; sunk: boolean }[];
	shots: number[];
	hits: number;
}

export interface BoardWalkLayout {
	rolls: number[];
	path: number[];
	board: number[];
}

export interface FishingLayout {
	casts: { fish: string | null; coins: number }[];
}

export interface WordBankLayout {
	words: string[];
	letters: string[];
	complete: number[];
}

export interface WordSearchLayout {
	grid: string[];
	targets: { word: string; coins: number; at: number[] | null }[];
}

export interface PumpkinLayout {
	pumpkins: ({ kind: 'ghost' } | { kind: 'candy'; candy: number } | { kind: 'golden' })[];
	candy: number;
}

export interface PotTicketLayout {
	instant: number;
}

export interface DuelLayout {
	opponent: string;
	mine: number[];
	theirs: number[];
	won: boolean;
}

export type Board = 'coins' | 'level';

/** See backend/src/routes/leaderboard.rs */
export interface LeaderboardDto {
	by: Board;
	rows: { rank: number; username: string; level: number; title: string | null; value: number }[];
	/** The viewer's place, top 50 or not; null for a guest */
	me: { rank: number; value: number } | null;
}

/** Today's shared pot, see backend/src/routes/pot.rs */
export interface PotDto {
	amount: number;
	entries: number;
	mine: number;
	draws_at: string;
	last: { day: string; winner: string | null; amount: number } | null;
}

export type DuelStatus = 'open' | 'accepted' | 'declined' | 'cancelled' | 'expired';

export interface DuelDto {
	id: number;
	challenger: string;
	opponent: string;
	stake: number;
	status: DuelStatus;
	created_at: string;
	winner: string | null;
}

export interface SnowballLayout {
	bumps: number[];
	sizes: number[];
}
