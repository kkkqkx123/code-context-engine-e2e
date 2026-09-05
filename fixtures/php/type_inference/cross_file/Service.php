<?php

require_once 'User.php';

use App\Models\User;
use App\Models\loadUser;

function renderGreeting(User $user): string {
    return $user->greet();
}

$user = loadUser("Alice");
echo renderGreeting($user) . "\n";
echo $user->greet() . "\n";
