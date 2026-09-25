export interface Session {
  id: string;
  user: string;
  createdAt: number;
}

export interface SessionRepository {
  save(session: Session): Promise<void>;
  find(id: string): Promise<Session | undefined>;
}

export class SessionService {
  constructor(private readonly repo: SessionRepository) {}

  create(user: string): Session {
    const session: Session = {
      id: Math.random().toString(36).slice(2),
      user,
      createdAt: Date.now(),
    };
    void this.repo.save(session);
    return session;
  }

  async load(id: string): Promise<Session | undefined> {
    return this.repo.find(id);
  }

  private validate(session: Session): boolean {
    return session.user.length > 0 && Number.isFinite(session.createdAt);
  }
}

export function formatSession(session: Session): string {
  return `${session.user}@${session.id}`;
}
