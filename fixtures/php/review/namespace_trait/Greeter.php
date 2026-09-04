<?php

namespace App\Services;

use App\Models\User;

class Greeter
{
    public function render(User $user): string
    {
        return $user->greet() . " @ " . $user->touch();
    }
}

$user = new User("Alice", 30);
$greeter = new Greeter();
echo $greeter->render($user) . "\n";
