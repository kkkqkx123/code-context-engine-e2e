<?php
use Foo\Bar;
use Qux\{Quux, Corge as C};
use function Acme\helper;
use const Acme\VERSION;
use Vendor\Package as VP;

class Service {
    use Loggable;
    use Cacheable, Retryable;

    public function handle(): void {
        $w = new Widget();
        Vendor\Package::create();
        $this->handle();
        parent::boot();
        self::version();
        $this->cache?->get();
        $this->db->table()->first();
    }
}
