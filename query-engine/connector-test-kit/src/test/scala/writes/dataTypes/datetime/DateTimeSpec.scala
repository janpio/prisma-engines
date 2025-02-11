package writes.dataTypes.datetime

import org.scalatest.{FlatSpec, Matchers}
import util._

class DateTimeSpec extends FlatSpec with Matchers with ApiSpecBase {

  val project = ProjectDsl.fromString {
    """|model Person {
       | id   String   @id @default(cuid())
       | name String   @unique
       | born DateTime
       |}"""
  }

  override def beforeEach(): Unit = {
    database.setup(project)
  }

  // FIXME: this panics the rust code. Let's fix that at some point.
  "Using a date before 1970" should "work" taggedAs IgnoreSQLite in {
    server.query(s"""mutation {createPerson(data: {name: "First", born: "1969-01-01T10:33:59Z"}){name}}""", project)
    val res = server.query(s"""query {person(where:{name: "First"}){name, born}}""", project)
    res.toString should be("""{"data":{"person":{"name":"First","born":"1969-01-01T10:33:59.000Z"}}}""")
  }

  "Using milliseconds in a date before 1970" should "work" taggedAs IgnoreSQLite in {
    server.query(s"""mutation {createPerson(data: {name: "Second", born: "1969-01-01T10:33:59.828Z"}){name}}""", project)
    val res = server.query(s"""query {person(where:{name: "Second"}){name, born}}""", project)
    res.toString should be("""{"data":{"person":{"name":"Second","born":"1969-01-01T10:33:59.828Z"}}}""")
  }

  "Using a date after 1970" should "work" in {
    server.query(s"""mutation {createPerson(data: {name: "Third", born: "1979-01-01T10:33:59Z"}){name}}""", project)
    val res = server.query(s"""query {person(where:{name: "Third"}){name, born}}""", project)
    res.toString should be("""{"data":{"person":{"name":"Third","born":"1979-01-01T10:33:59.000Z"}}}""")
  }

  "Using milliseconds in a date after 1970" should "work" in {
    server.query(s"""mutation {createPerson(data: {name: "Fourth", born: "1979-01-01T10:33:59.828Z"}){name}}""", project)
    val res = server.query(s"""query {person(where:{name: "Fourth"}){name, born}}""", project)
    res.toString should be("""{"data":{"person":{"name":"Fourth","born":"1979-01-01T10:33:59.828Z"}}}""")
  }

  // https://tools.ietf.org/html/rfc3339 doesn't support 5-digit years. Therefore Rust date libraries will give a parse
  // error here.
  "Using a date after 10000" should "work" ignore {
    server.query(s"""mutation {createPerson(data: {name: "Fifth", born: "11979-01-01T10:33:59Z"}){name}}""", project)
    val res = server.query(s"""query {person(where:{name: "Fifth"}){name, born}}""", project)
    res.toString should be("""{"data":{"person":{"name":"Fifth","born":"11979-01-01T10:33:59.000Z"}}}""")
  }

  "Using milliseconds in a date after 10000" should "work" ignore {
    server.query(s"""mutation {createPerson(data: {name: "Sixth", born: "11979-01-01T10:33:59.828Z"}){name}}""", project)
    val res = server.query(s"""query {person(where:{name: "Sixth"}){name, born}}""", project)
    res.toString should be("""{"data":{"person":{"name":"Sixth","born":"11979-01-01T10:33:59.828Z"}}}""")
  }

  // Everything Before Christ is a mess anyway calendar-wise.

//  "Using a date before 0" should "work" in {
//    server.query(s"""mutation {createPerson(data: {name: "Seventh", born: "-0500-01-01T10:33:59Z"}){name}}""", project)
//    val res = server.query(s"""query {person(where:{name: "Seventh"}){name, born}}""", project)
//    res.toString should be("""{"data":{"person":{"name":"Seventh","born":"-0500-01-01T10:33:59Z"}}}""")
//  }
//
//  "Using milliseconds in a date before 0" should "work" in {
//    server.query(s"""mutation {createPerson(data: {name: "Eighth", born: "-0500-01-01T10:33:59.828Z"}){name}}""", project)
//    val res = server.query(s"""query {person(where:{name: "Eighth"}){name, born}}""", project)
//    res.toString should be("""{"data":{"person":{"name":"Eighth","born":"-0500-01-01T10:33:59.828Z"}}}""")
//  }

}
