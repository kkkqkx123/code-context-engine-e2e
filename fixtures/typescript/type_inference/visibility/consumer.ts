import { Visibility } from './visibility';

export function consumePublic(v: Visibility): string {
    return v.getPublic();
}

export function consumeDescribe(v: Visibility): string {
    return v.describe();
}
