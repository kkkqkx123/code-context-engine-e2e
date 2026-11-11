import a.{b, c => d}
import util._
import conf.{host, port => servicePort}
import store.{backend => _}

object App {
  def main(args: Array[String]): Unit = {
    val svc = new Service
    svc.start()
    val total = 1 + 2
    ()
  }
}
