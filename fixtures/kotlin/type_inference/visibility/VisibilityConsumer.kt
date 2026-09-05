package app.service

import app.models.Visibility

fun consumePublic(v: Visibility): String = v.getPublic()

fun consumeDescribe(v: Visibility): String = v.describe()
