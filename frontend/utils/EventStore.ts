import { CalendarEvent } from "./CalendarEvent";
import dayjs from "./dayjs";

// TODO: implement overrides
export class EventEntry {
    eventID: string;
    startTime: dayjs.Dayjs;
    endTime: dayjs.Dayjs;
}

class EventIteratorContext {
    private store: EventStore;
    private startAt: number;
    private endAt: number;

    constructor(store: EventStore) {
        this.store = store;
        this.startAt = 0;
        this.endAt = -1;
    }

    // Sets the starting point. Can be a date or event index. Inclusive. Default 0.
    from(startPoint: dayjs.Dayjs | number) {
        if (typeof startPoint != "number") {
            this.startAt = this.store.findIndexAfter(startPoint);
        } else {
            this.startAt = startPoint;
        }
        return this;
    }

    // Sets the ending point. Can be a date or event index. Exclusive. Default -1 (end of the list)
    to(endPoint: dayjs.Dayjs | number) {
        if (typeof endPoint != "number") {
            this.endAt = this.store.findIndexBefore(endPoint);
        } else {
            this.endAt = endPoint;
        }
        return this;
    }

    // Sets the ending point by adding a number to the starting point.
    next(count: number) {
        this.endAt = this.startAt + count;
        return this;
    }

    *[Symbol.iterator]() {
        let startIndex = this.startAt;

        let endIndex: number;
        if (this.endAt < 0) {
            endIndex = this.store.entries.length;
        } else {
            endIndex = this.endAt;
        }

        if (startIndex < endIndex) {
            throw new RangeError("Invalid search bounds");
        }

        for (let i = startIndex; i < endIndex; i++) {
            // TODO: implement overrides
            yield this.store.data.get(this.store.entries[i].eventID)!;
        }
    }
}

export class EventStore {
    // Chronological list of entries
    entries: EventEntry[];
    data: Map<string, CalendarEvent>;

    constructor() {
        // TODO: example data
        // TODO^2: fetch from API
        this.entries = [];
        this.data = new Map();
    }

    findIndexBefore(timestamp: dayjs.Dayjs): number {
        // TODO
        throw new Error("Searching by timestamp not implemented");
    }

    findIndexAfter(timestamp: dayjs.Dayjs): number {
        // TODO
        throw new Error("Searching by timestamp not implemented");
    }

    iter() {
        return new EventIteratorContext(this);
    }
}
