<?php

class StringUtils
{
    public static function reverse(string $input): string
    {
        return strrev($input);
    }

    public static function wordCount(string $input): int
    {
        return str_word_count($input);
    }
}
