package app.service

import app.models.Visibility

object VisibilityConsumer {
  def consumePublic(v: Visibility): String = v.getPublic()

  def consumeDescribe(v: Visibility): String = v.describe()
}
